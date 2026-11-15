//! Zray for Zed: ABAP with the Zray language server (`zray lsp`).
//!
//! The extension only finds, verifies and starts the server:
//!
//! 1. `lsp.zray.binary.path` in Zed's settings (developers, companies that distribute `zray` themselves);
//! 2. `zray` on the `PATH`;
//! 3. the release named by a manifest (`lsp.zray.settings.manifestUrl`, default [`DEFAULT_MANIFEST`], the latest
//!    GitHub release; `""` turns downloading off; JSON:
//!    `{ "version": "0.1.0", "platforms": { "windows-x86_64": { "url": "...zip", "sha256": "<of the binary>",
//!    "binary": "zray.exe" } } }`): downloaded once into the extension's folder, its SHA-256 checked (a mismatch
//!    deletes it and fails loudly), older versions removed;
//! 4. offline: the newest version downloaded before.
//!
//! It also provides the MCP server `zray` for Zed's agent (`zray mcp`, the same binary): the analysis of the
//! project's ABAP as tools (objects, code, findings, dependencies, the context of a statement, fixes).

use sha2::{Digest, Sha256};
use std::fs;
use zed_extension_api::{
    self as zed, http_client::HttpMethod, http_client::HttpRequest, http_client::RedirectPolicy, serde_json,
    settings::{ContextServerSettings, LspSettings},
    ContextServerConfiguration, ContextServerId, LanguageServerId, LanguageServerInstallationStatus, Project, Result,
};

const SERVER: &str = "zray";
const PREFIX: &str = "zray-";
/// The release manifest when `manifestUrl` is not set: the `manifest.json` of this repository's latest GitHub
/// release (the zray binaries and the manifest are published there).
const DEFAULT_MANIFEST: &str = "https://github.com/plotnetan/zray-zed/releases/latest/download/manifest.json";
const NOT_FOUND: &str = "put zray on the PATH, or set \"lsp\": { \"zray\": { \"binary\": { \"path\": \"...\" } } } \
                         (or \"settings\": { \"manifestUrl\": \"...\" }) in Zed's settings";
/// The binary the language server started with, kept in the extension's work folder for the MCP server: Zed may start
/// the MCP server before any ABAP file is open (no worktree to look the binary up in), and the extension cannot check a
/// path outside its folder (WASI sandbox).
const LAST_BINARY: &str = "last-binary.txt";

/// The standard installation on Windows, `<LOCALAPPDATA>\zray\zray.exe`, from the extension's work folder (the
/// folder above `Zed`); None elsewhere or when it cannot be told.
fn standard_install() -> Option<String> {
    if zed::current_platform().0 != zed::Os::Windows {
        return None;
    }
    let dir = std::env::current_dir().ok()?;
    let local = dir.ancestors().find(|a| a.join("Zed").is_dir() && a.file_name().is_some_and(|n| n.eq_ignore_ascii_case("Local")))
        .or_else(|| dir.ancestors().nth(4))?;
    Some(local.join("zray").join("zray.exe").to_string_lossy().replace('\\', "/"))
}

fn remember(bin: &str) {
    if fs::read_to_string(LAST_BINARY).ok().as_deref() != Some(bin) {
        let _ = fs::write(LAST_BINARY, bin);
    }
}

struct Zray {
    /// The binary found in this session (no second download check per worktree).
    cached: Option<String>,
}

fn platform_key() -> Result<String> {
    let (os, arch) = zed::current_platform();
    let os = match os {
        zed::Os::Windows => "windows",
        zed::Os::Mac => "macos",
        zed::Os::Linux => "linux",
    };
    let arch = match arch {
        zed::Architecture::X8664 => "x86_64",
        zed::Architecture::Aarch64 => "aarch64",
        zed::Architecture::X86 => return Err("Zray needs a 64-bit system".into()),
    };
    Ok(format!("{os}-{arch}"))
}

fn binary_name() -> &'static str {
    match zed::current_platform().0 {
        zed::Os::Windows => "zray.exe",
        _ => "zray",
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|b| format!("{b:02x}")).collect()
}

/// The newest version downloaded before (offline start).
fn newest_local() -> Option<String> {
    let mut v: Vec<String> = fs::read_dir(".")
        .ok()?
        .flatten()
        .filter_map(|e| e.file_name().to_str().map(String::from))
        .filter(|n| n.starts_with(PREFIX))
        .collect();
    v.sort_by_key(|n| version_key(&n[PREFIX.len()..]));
    let dir = v.pop()?;
    let bin = format!("{dir}/{}", binary_name());
    fs::metadata(&bin).ok().map(|_| bin)
}

fn version_key(v: &str) -> Vec<u64> {
    v.split(['.', '-']).map(|p| p.parse().unwrap_or(0)).collect()
}

impl Zray {
    /// Download the release of the manifest (once per version) and check its SHA-256.
    fn download(&self, id: &LanguageServerId, manifest_url: &str) -> Result<String> {
        zed::set_language_server_installation_status(id, &LanguageServerInstallationStatus::CheckingForUpdate);
        let resp = HttpRequest::builder()
            .method(HttpMethod::Get)
            .url(manifest_url)
            .redirect_policy(RedirectPolicy::FollowAll)
            .build()?
            .fetch()
            .map_err(|e| format!("cannot read the Zray release manifest {manifest_url}: {e}"))?;
        let m: serde_json::Value = serde_json::from_slice(&resp.body)
            .map_err(|e| format!("the Zray release manifest {manifest_url} is not JSON: {e}"))?;
        let version = m["version"].as_str().ok_or("the release manifest has no version")?.to_string();
        let key = platform_key()?;
        let p = &m["platforms"][&key];
        let url = p["url"]
            .as_str()
            .ok_or_else(|| format!("no Zray release for {key} in the manifest"))?;
        let sha = p["sha256"].as_str().ok_or("the release manifest has no sha256")?.to_ascii_lowercase();
        let name = p["binary"].as_str().unwrap_or(binary_name());
        let dir = format!("{PREFIX}{version}");
        let bin = format!("{dir}/{name}");
        let verified = |path: &str| fs::read(path).map(|b| sha256_hex(&b) == sha).unwrap_or(false);
        if !verified(&bin) {
            zed::set_language_server_installation_status(id, &LanguageServerInstallationStatus::Downloading);
            let _ = fs::remove_dir_all(&dir);
            let kind = if url.ends_with(".zip") {
                zed::DownloadedFileType::Zip
            } else if url.ends_with(".tar.gz") || url.ends_with(".tgz") {
                zed::DownloadedFileType::GzipTar
            } else {
                zed::DownloadedFileType::Uncompressed
            };
            let target = if kind == zed::DownloadedFileType::Uncompressed { bin.clone() } else { dir.clone() };
            if kind == zed::DownloadedFileType::Uncompressed {
                fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
            }
            zed::download_file(url, &target, kind).map_err(|e| format!("cannot download {url}: {e}"))?;
            if !verified(&bin) {
                let _ = fs::remove_dir_all(&dir);
                return Err(format!(
                    "the downloaded Zray server does not match its SHA-256 ({url}): removed; nothing was started"
                ));
            }
            zed::make_file_executable(&bin)?;
            // older versions
            if let Ok(entries) = fs::read_dir(".") {
                for e in entries.flatten() {
                    if let Some(n) = e.file_name().to_str() {
                        if n.starts_with(PREFIX) && n != dir {
                            let _ = fs::remove_dir_all(n);
                        }
                    }
                }
            }
        }
        zed::set_language_server_installation_status(id, &LanguageServerInstallationStatus::None);
        Ok(bin)
    }

    fn binary(&mut self, id: &LanguageServerId, worktree: &zed::Worktree) -> Result<String> {
        let settings = LspSettings::for_worktree(SERVER, worktree).ok();
        if let Some(path) = settings.as_ref().and_then(|s| s.binary.as_ref()).and_then(|b| b.path.clone()) {
            // remembered for the MCP server, which has no worktree to look in
            self.cached = Some(path.clone());
            remember(&path);
            return Ok(path);
        }
        if let Some(path) = worktree.which("zray") {
            self.cached = Some(path.clone());
            remember(&path);
            return Ok(path);
        }
        if let Some(bin) = &self.cached {
            if fs::metadata(bin).is_ok() {
                return Ok(bin.clone());
            }
        }
        // not set: the default; set to "" (or blank): no download
        let manifest = match settings.as_ref().and_then(|s| s.settings.as_ref()).and_then(|s| s.get("manifestUrl")) {
            Some(v) => v.as_str().map(str::trim).filter(|u| !u.is_empty()).map(String::from),
            None => Some(DEFAULT_MANIFEST.to_string()),
        };
        let found = match manifest {
            Some(url) => match self.download(id, &url) {
                Ok(bin) => Ok(bin),
                // offline / host down: the version downloaded before
                Err(e) => newest_local().ok_or_else(|| format!("{e}. Or {NOT_FOUND}")),
            },
            None => newest_local().ok_or_else(|| format!("the Zray server (zray) was not found: {NOT_FOUND}")),
        };
        match found {
            Ok(bin) => {
                self.cached = Some(bin.clone());
                remember(&bin);
                Ok(bin)
            }
            Err(e) => {
                zed::set_language_server_installation_status(id, &LanguageServerInstallationStatus::Failed(e.clone()));
                Err(e)
            }
        }
    }
}

impl zed::Extension for Zray {
    fn new() -> Self {
        Zray { cached: None }
    }

    fn language_server_command(&mut self, id: &LanguageServerId, worktree: &zed::Worktree) -> Result<zed::Command> {
        let command = self.binary(id, worktree)?;
        let configured = LspSettings::for_worktree(SERVER, worktree).ok().and_then(|s| s.binary);
        let args = configured
            .as_ref()
            .and_then(|b| b.arguments.clone())
            .unwrap_or_else(|| vec!["lsp".into()]);
        let mut env: Vec<(String, String)> = configured
            .and_then(|b| b.env)
            .map(|e| e.into_iter().collect())
            .unwrap_or_default();
        // the server names itself in messages ("run <ZRAY_SELF> sap login")
        env.push(("ZRAY_SELF".into(), command.clone()));
        Ok(zed::Command { command, args, env })
    }

    /// The MCP server: `zray mcp` with the folder, tool sets and push permission of the context server's settings
    /// (`context_servers.zray.settings`: `folder`, `tools`, `allowPush`). Without a folder the server answers on
    /// the project the language server opened last.
    fn context_server_command(&mut self, id: &ContextServerId, project: &Project) -> Result<zed::Command> {
        let cs = ContextServerSettings::for_project(id.as_ref(), project).ok();
        let configured = cs.as_ref().and_then(|s| s.command.as_ref());
        // the configured command, else the binary the language server uses (this session, else the last one), else the
        // downloaded one; no existence check: a path outside the extension's folder is invisible to it
        let command = configured
            .and_then(|c| c.path.clone())
            .or_else(|| self.cached.clone())
            .or_else(|| fs::read_to_string(LAST_BINARY).ok().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()))
            .or_else(|| {
                // a downloaded binary is relative to the work folder: Zed needs it absolute
                newest_local().map(|b| {
                    std::env::current_dir().map(|d| d.join(&b).to_string_lossy().to_string()).unwrap_or(b)
                })
            });
        let args = match configured.and_then(|c| c.arguments.clone()) {
            Some(a) => a,
            None => {
                let s = cs.as_ref().and_then(|s| s.settings.clone()).unwrap_or_default();
                let mut a = vec!["mcp".to_string()];
                if let Some(f) = s.get("folder").and_then(|v| v.as_str()).filter(|f| !f.trim().is_empty()) {
                    a.push(f.to_string());
                }
                if let Some(t) = s.get("tools").and_then(|v| v.as_str()).filter(|t| !t.trim().is_empty()) {
                    a.push(format!("--tools={t}"));
                }
                if s.get("allowPush").and_then(|v| v.as_bool()).unwrap_or(false) {
                    a.push("--allow-push".into());
                }
                a
            }
        };
        let mut env: Vec<(String, String)> = configured
            .and_then(|c| c.env.clone())
            .map(|e| e.into_iter().collect())
            .unwrap_or_default();
        // not known yet (Zed starts the MCP server before the language server): the standard installation, found from
        // the extension's own work folder (`<LOCALAPPDATA>\Zed\extensions\work\zray` → `<LOCALAPPDATA>\zray`)
        let command = command.or_else(standard_install).unwrap_or_else(|| "zray".to_string());
        env.push(("ZRAY_SELF".into(), command.clone()));
        // what Zed is told to run, for a server that does not start
        let _ = fs::write("context-server-command.txt", format!("{command} {}\n", args.join(" ")));
        Ok(zed::Command { command, args, env })
    }

    fn context_server_configuration(
        &mut self,
        _id: &ContextServerId,
        _project: &Project,
    ) -> Result<Option<ContextServerConfiguration>> {
        Ok(Some(ContextServerConfiguration {
            installation_instructions: "Zray gives Zed's agent the analysis of your ABAP: objects, code search, \
                findings and their rules, dependencies (where used, call graph, table usage, impact), the context of a \
                statement, verified fixes, DDIC and CDS. Read-only unless you turn the actions on.\n\n\
                - **folder**: the SAP folder to answer on (default: the project the Zray language server opened \
                last; open an ABAP file first).\n\
                - **tools**: tool sets besides core: `impact`, `fixes`, `cds`, `actions` (default: all but actions; \
                `all,actions` adds analyze and pull).\n\
                - **allowPush**: with the actions set, also `push` to SAP (the plan first; it pushes only when you \
                confirm, into the transport request you name)."
                .to_string(),
            settings_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "folder": { "type": "string", "description": "The SAP folder (default: the project the language server opened last)" },
                    "tools": { "type": "string", "description": "Tool sets besides core: impact, fixes, cds, actions (comma separated)" },
                    "allowPush": { "type": "boolean", "description": "Offer push to SAP (needs the actions set)" }
                }
            })
            .to_string(),
            default_settings: "{\n  \"folder\": \"\",\n  \"tools\": \"all\",\n  \"allowPush\": false\n}".to_string(),
        }))
    }

    fn language_server_initialization_options(
        &mut self,
        _id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<serde_json::Value>> {
        Ok(LspSettings::for_worktree(SERVER, worktree)
            .ok()
            .and_then(|s| s.initialization_options))
    }

    fn language_server_workspace_configuration(
        &mut self,
        _id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<serde_json::Value>> {
        Ok(LspSettings::for_worktree(SERVER, worktree)
            .ok()
            .and_then(|s| s.settings)
            .map(|s| serde_json::json!({ "zray": s })))
    }
}

zed::register_extension!(Zray);
