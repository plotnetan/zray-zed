# Zray for Zed

ABAP, CDS and RAP behavior definitions in [Zed](https://zed.dev): diagnostics as you type, verified fixes, navigation, and a
connection to your SAP development system (ADT). The extension (MIT) starts the Zray language server
(`zray lsp`), a local process of Zed: no server to install, nothing leaves your computer except the
requests to your own SAP system.

## Install

Until Zray is published in Zed's extension registry, it is installed as a development extension (this needs
[Rust](https://rustup.rs) and git; Zed compiles the extension):

1. Clone this repository: `git clone https://github.com/plotnetan/zray-zed`.
2. Command palette → `zed: install dev extension` → the cloned folder. The first time Zed compiles the extension and
   the grammars (about 2 minutes, no progress shown).
3. The server, `zray`: the extension downloads the latest release of this repository
   (`https://github.com/plotnetan/zray-zed/releases/latest/download/manifest.json`), checks its SHA-256 and keeps it
   (offline: the last one downloaded). Nothing to set up. To use a `zray` of your own instead, put it on the `PATH`,
   or name it in Zed's settings (`ctrl-,`):

   ```json
   "lsp": {
     "zray": {
       "binary": { "path": "C:/Users/<you>/AppData/Local/zray/zray.exe" }
     }
   }
   ```

   `"settings": { "manifestUrl": "…" }` names another release manifest (a company's own copy).
4. Connect a folder to SAP (next section), or open a folder that already holds ABAP sources (an abapGit
   repository, an ETL import). **Trust the folder** when Zed asks: in restricted mode Zed starts no language server.

Sources must be `.abap` files. An ETL export with `.txt` sources needs `.zed/settings.json` in the folder:
`{ "file_types": { "ABAP": ["txt"] } }`. Do not point `binary.path` at a build output: a running server locks the
`.exe` on Windows.

## Connect to SAP: `zray.start`

Reading is the default (sources, package contents, DDIC). The only thing that writes to SAP is **push**, and only
when you start it and choose where the change goes.

1. **Create an empty folder** (one per SAP system, e.g. `C:\sap\dev`), open it in Zed (File → Open Folder) and trust it.
2. **Create the file `zray.start`** in it (project panel → New File). Zray:
   - checks that the folder is empty. If it is not, the file says so and nothing else is written;
   - writes **`README.md`**, the full guide: first connection, daily work, the folder's files, several systems,
     production, AI assistants and troubleshooting;
   - writes **`AGENTS.md`**, the same steps for an AI assistant. Zed's agent reads it, so it can do the connection
     with you. It never handles the password;
   - turns `zray.start` into a **form**: `url`, `client`, `user`, `production`, `language`, with the status of
     each step below them. When Zed's settings name a system, the form starts with it.
3. **Fill in the form and save it** (Ctrl+S). Zray:
   - records the system in the folder (`.zray/mirror.json`);
   - checks that the host answers, then tests ADT;
   - opens the Windows password dialog when there is no valid password, and stores the password in the Credential
     Manager;
   - reads the package list into **`packages.sappkg`** and opens it.

   Each step shows ✓ / … / ✗ in the form. An error comes with what to do (URL and port, VPN, certificate, user or
   password, ADT not active, authorizations, single sign-on), and its details go to **`zray.log`**.
4. **Connect packages**: Ctrl+. on a package line of `packages.sappkg` → *connect package*. Or create a folder named
   like the package and answer the question.

Ctrl+. in `zray.start`: *connect now*, *log in to SAP again*, open the guide, the AI instructions, the log or the
catalog.

**A folder keeps its own system.** Once connected, the system in `.zray/mirror.json` is used, whatever Zed's settings
say. Zed's `sap` settings only give the system of a folder that has none yet, or the properties file to use for that
same system. So folders of several systems can be open, each in its own window. A folder whose packages are connected
refuses another system: create another folder.

From then on `ctrl-.` offers:

- **Zray: refresh from SAP** — only what changed; **refresh package X from SAP** — the current file's package;
- **Zray: push this file to SAP** / **push all local changes to SAP**: your saved changes are written, then
  activated. A prompt lists your transport requests as buttons (the one you used last comes first). A change made in
  SAP since your pull is not overwritten unless you confirm. Activation errors are shown with file and line;
- **Zray: log in to SAP** — the Windows credential dialog; **test the SAP connection**;
- **Zray: analyze the whole package** — every rule, including the ones across objects (calls between programs,
  LUW, unused objects …): progress in the status bar (cancellable), the cross-object findings as diagnostics, and a
  report (with a link to the printable executive report).

In a terminal, the same is `zray sap login`, `zray sap packages --out <folder>` and `zray sap pull <PACKAGE> --out
<folder>`, with `--url --client --user` or `--system <file.properties>`. A bare host means `http://host:8000`.

## The whole system: `analysis.sapanalysis`

In a connected folder, Ctrl+. in `zray.start` or on the header of `packages.sappkg` → *set up a full system
analysis* writes `analysis.sapanalysis` (six steps, one line each) and `analysis.json` (the choices). Ctrl+. on a
step runs it; the steps read every customer object (sources, DDIC, CDS, messages, texts), what they use from SAP,
the where-used lists, transports and logs, the configuration tables, the usage data, and then analyze it all. Each
line shows ● done, ○ not run (with an estimate), ◐ stopped (continues), ✗ failed, ⟳ out of date.

Zray asks before it reads: once per system how (the fetcher classes `ZCL_FETCHER_*` in `$TMP`, or SQL with
nothing installed), once per run whether, and on a production system before every step ("Read PRODUCTION", always
SQL). Details: the folder's `README.md`, section 3.

## What you get while editing

- **Diagnostics** as you type: parse errors and the per-file rules, with the whole program as context (TOP
  include, other includes, class sections) and the folder's DDIC (`indexed-ddic.json`).
- **Code actions** (`ctrl-.`): the verified fixes for the statement under the cursor (SAFE ones preferred), and
  "Apply all safe fixes in this file".
- **Go to definition** (`F12`) also on members (`lo->attr`, `zcl_x=>const`, `ls_x-comp`), **hover** (declarations;
  DDIC fields and tables; keyword documentation of the system's release), **outline**, **workspace symbols**
  (`ctrl-t`), **completion** (keywords, the program's names, and after `->` `=>` `~` `-` the members).

## CDS

CDS sources (`.ddls`, `.ddlx`, `.dcls`, `.srvd`; abapGit `.asddls` …) open as **ABAP CDS**: highlighting and outline,
the CDS checks while typing (access control, performance, unused associations, annotations, keys of to-one
associations …; "unused view" needs the package analysis), the fix for an unused association, go to definition and
hover on data sources, associations and fields (through other views into the DDIC), completion of fields after `.`,
of annotations after `@` and of views and tables after `from` / `join` / `to`. In ABAP, hover on a CDS view of a
`SELECT` shows its elements.

## Behavior definitions (RAP)

Behavior definitions (`.bdef`, abapGit `.bdef.asbdef`) open as **ABAP BDEF**: highlighting and outline (entities,
actions, determinations, validations, associations, fields), the RAP checks while typing (no authorization master,
draft lock master without total etag, an unmanaged saver without SAVE / CLEANUP), go to definition and hover on the
entities (the CDS view), their elements and associations, the persistent / draft / mapping tables and their fields
(DDIC), the behavior pool class; completion of the entity's elements and of BDEF keywords.

## AI assistants (MCP)

The same binary is an MCP server: `zray mcp <folder>`. Zed's agent panel, Claude Code, Cursor and other AI clients then
answer from the analysis: objects, routines, source, code search, findings and their rules, DDIC, dependencies
(where used, call graph, table usage, related, influenced by), a syntax check of code they write. Read-only: nothing changes SAP or the files.

**In Zed it is built in:** the extension provides the MCP server **zray**. Open the agent panel → its settings →
the server list → turn on *zray*. Settings (`context_servers.zray.settings`): `folder` (default: the project
the Zray language server opened last; open an ABAP file first), `tools` (`all`, or e.g. `core,impact`; add
`actions` for analyze / pull), `allowPush` (with `actions`: push to SAP, plan first).

**Inline assistant: `@aic`.** Zed's inline assistant cannot call tools (MCP), but it takes files as context. Put the
cursor on a statement → **ctrl-.** → *AI context of this statement (@aic for the inline assistant)*. Zray writes
`.zray/aic.md`, and a message says when it is ready. It holds the statement, its routine and signature, every name
it uses resolved (variables with their types, DDIC fields, called methods), the findings with their fixes, the
verified fixes as before / after, the callers, and the source with the statement marked. Then open the inline
assistant (**ctrl-enter**), type **`@aic`**, and ask ("fix the finding", "explain this", "rewrite with NEW"). Save
the file first: the context reads the saved source. From a terminal: `zray aic <file> --line N`.

**Other clients:** **ctrl-.** → *use this folder in other AI assistants (MCP setup …)* writes `.zray/mcp-setup.md` with
the settings below for this folder and this binary filled in, and opens it.

Zed by hand (not needed with the extension; `settings.json`):

```json
"context_servers": {
  "zray": {
    "command": { "path": "C:/Users/<you>/AppData/Local/zray/zray.exe", "args": ["mcp", "C:/dev"] }
  }
}
```

Claude Code: `claude mcp add zray -- C:/Users/<you>/AppData/Local/zray/zray.exe mcp C:/dev`

`--tools=core` offers fewer tools (less context for the model). `--tools=all,actions` adds `analyze` and `pull`;
with `--allow-push` also `push` (it shows the plan first and pushes only when confirmed, into the request you name). The findings across objects are those of the last
"analyze the whole package"; files changed since then get their per-file findings computed on the call.

## The project's configuration: `zray.json`

Every SAP folder (a folder connected to SAP, or with an ETL extract) gets three files, written when they are missing:

- **`zray.json`**: the rules, fixes, pretty print and analysis of this folder, shared by the team (commit it) and
  read by the editor, the MCP server and `zray aic`. Zed completes and checks it while you type (the schema
  `.zray/zray.schema.json`). A key you change overrides Zed's `lsp.zray` setting; a key left at its default
  leaves Zed's setting in force. Changes apply at once. **Delete the file to reset it**: it comes back with the
  defaults.
- **`ZRAY.md`**: a guide for the developer (the actions, `@aic`, the MCP server, every key). Edit it as you like;
  it is written again only when deleted.

```json
{
  "$schema": "./.zray/zray.schema.json",
  "rules": {
    "minSeverity": "info",
    "disabled": ["PERF-004"],
    "enabled": [],
    "severity": { "BUG-002": "high" },
    "exclude": ["$tmp/*", "ztest_*"]
  },
  "fixes": { "maxSafety": "check", "never": ["FIX-CREATE-OBJECT"], "settings": { "FIX-FAE-GUARD.else": "none" } },
  "prettyPrint": { "keywordCase": "upper" },
  "analysis": { "release": 757, "afterPull": "first" }
}
```

| Key | Meaning |
|---|---|
| `rules.minSeverity` | findings below it are hidden (`info` … `critical`) |
| `rules.disabled` / `rules.enabled` | rules hidden / shown although off by default (`CORE-006`) |
| `rules.severity` | a rule's severity in this project |
| `rules.exclude` | files without findings: patterns on the path or the file name |
| `fixes.maxSafety` | the least safe fix offered: `safe`, `check`, `manual` |
| `fixes.never` | fixes never offered, nor applied by "apply all safe fixes" |
| `fixes.settings` | per-fix options `<FIX-ID>.<name>` |
| `prettyPrint.keywordCase` | `auto`, `upper`, `lower`: keywords of completions and fixes |
| `analysis.release` | ABAP release of the target system; `null`: the mirror's |
| `analysis.afterPull` | `first`, `always`, `never`: analyze the package after a pull |

### Production systems

Say `production = yes` in `zray.start`, or add `role=production` to the system's properties file
(`~/.abap-analysis/<system>.properties`; the folder's system is found by its URL and client). Then:

- a push from the editor asks again ("Push to PRODUCTION"); `zray sap push` needs `--production`
- the MCP server tells the AI it is production and refuses `push`
- the status bar shows **⛔ PRODUCTION SAP system · PRD client 100 (url)** for as long as the folder is open (a
  progress of the language server that does not end; Zed applies themes from the user settings only, so a folder's
  colors cannot change)

The other roles (`quality`, `development`, `sandbox`) change nothing yet.

## Languages

Zray speaks **English** (`en`), **German** (`de`), **French** (`fr`), **Spanish** (`es`), **Japanese**
(`ja`), **Mandarin** (`zh`, simplified Chinese) and **Hebrew** (`he`). The language covers the `zray.start` form and its
status, the folder's `README.md`, the findings and the parser's remarks in Problems and on hover, and the actions of
the form.

- **Per folder:** `language = de` in `zray.start`, then save. The folder keeps it (`.zray/user.json`), the form
  and `README.md` are written again in that language, and every open file is analyzed again. The form's line lists
  the languages Zray supports.
- **For every folder:** the setting `language` below (`"de"`). A folder's own choice wins.
- **Hebrew is poorly viewed:** Zed has no right-to-left support and shows Hebrew reversed. It stays available; the
  form and a message say so, in English.

A finding the catalog has no exact text for shows the rule's title in the language above the English message.
`AGENTS.md` stays English (AI assistants follow it best) and tells the assistant to answer in your language. The
keys of the form (`url`, `client` …), ABAP keywords, file names and SAP terms stay as they are.

## Settings (`initialization_options` or `settings` of `lsp.zray`)

| key | default | meaning |
|---|---|---|
| `release` | the mirror's, else 750 | ABAP release fixes generate syntax for (`758` or `"7.58"`) |
| `minSeverity` | `info` | hide findings below `info` / `low` / `medium` / `high` / `critical` |
| `disabledRules` | `[]` | rule ids to hide |
| `enabledRules` | `[]` | rule ids to show although off by default (`CORE-006` is off by default) |
| `reference` | `ABAP_REFERENCE` | SAP reference data folder (`<release>_JSON`) |
| `keywordCase` | `auto` | completion of keywords: `auto` (as typed), `upper`, `lower` |
| `cache` / `cacheDir` | `true` / `%LOCALAPPDATA%\zray\lsp\<hash>` | the facts cache (never inside the project) |
| `sap` | — | `{ "url", "client", "user" }` or `{ "system": "<file.properties>" }`: the system of a folder that has none yet (the folder's own `.zray/mirror.json` wins), and the properties file used for that same system |
| `analyzeAfterPull` | `first` | analyze the whole package in the background after connecting it (`first`), also after each refresh that changed files (`always`), or `never`; a command you start stops it, and it runs again afterwards |
| `language` | `en` | the language of zray's texts and findings: `en`, `de`, `fr`, `es`, `ja`, `zh` or `he` (shown reversed in Zed); `language` in a folder's `zray.start` wins |
| `manifestUrl` | the latest release of this repository | (`settings` only) the release manifest the extension downloads `zray` from when it is neither set in `binary.path` nor on the `PATH`; `""` turns downloading off |

Severity mapping: critical → error, high / medium → warning, low → information, info → hint.

## License and grammars

The extension is MIT (`LICENSE`); the language server it starts is proprietary Zray software. Highlighting,
brackets, indentation and outline come from Zray's own grammars, `tree-sitter-zray` (MIT): `abap`, `abap_cds`,
`abap_bdef`; their queries are `languages/*/*.scm`.
