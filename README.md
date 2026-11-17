# Zray for Zed

ABAP, CDS and RAP behavior definitions in [Zed](https://zed.dev): diagnostics as you type, verified fixes, navigation,
and a connection to your SAP development system (ADT). The extension (MIT) starts the Zray language server
(`zray lsp`) as a local process inside Zed. There is no server to install, and nothing leaves your computer except
the requests to your own SAP system.

## Install

Until Zray is published in Zed's extension registry, install it as a development extension. This needs
[Rust](https://rustup.rs) and git, because Zed compiles the extension itself.

1. Clone this repository: `git clone https://github.com/plotnetan/zray-zed`.
2. Open the command palette, run `zed: install dev extension` and pick the cloned folder. The first time, Zed compiles
   the extension and its grammars; this takes about 2 minutes and shows no progress.
3. The server, `zray`, needs no setup. The extension downloads the latest release of this repository
   (`https://github.com/plotnetan/zray-zed/releases/latest/download/manifest.json`), verifies its SHA-256 and keeps
   a copy, so it also works offline with the last download. To use your own `zray` instead, put it on the `PATH` or
   set it in Zed's settings (`ctrl-,`):

   ```json
   "lsp": {
     "zray": {
       "binary": { "path": "C:/Users/<you>/AppData/Local/zray/zray.exe" }
     }
   }
   ```

   To download from a different release manifest (for example, your company's own copy), set
   `"settings": { "manifestUrl": "…" }`.
4. Connect a folder to SAP (see the next section), or open a folder that already contains ABAP sources, such as an
   abapGit repository. **Trust the folder** when Zed asks: in restricted mode, Zed does not start language servers.

Sources must be `.abap` files. Don't point `binary.path` at a build output: on Windows, a running server locks the
`.exe`.

## Connect to SAP: `zray.start`

By default, Zray only reads from SAP (sources, package contents, DDIC). The only thing that writes to SAP is
**push**, and only when you start it and choose where the change goes.

1. **Create an empty folder** (one per SAP system, e.g. `C:\sap\dev`), open it in Zed (File → Open Folder) and trust
   it.
2. **Create a file named `zray.start`** in it (project panel → New File). Zray then:
   - checks that the folder is empty. If it isn't, the file says so and nothing else is written;
   - writes **`README.md`**, the full guide: first connection, daily work, the folder's files, multiple systems,
     production, AI assistants and troubleshooting;
   - writes **`AGENTS.md`**, the same steps for an AI assistant. Zed's agent reads it and can walk you through the
     connection. It never handles your password;
   - turns `zray.start` into a **form** with the fields `url`, `client`, `user`, `production` and `language`, and the
     status of each step below them. If Zed's settings specify a system, the form is pre-filled with it.
3. **Fill in the form and save it** (`ctrl-s`). Zray then:
   - records the system in the folder (`.zray/mirror.json`);
   - checks that the host responds, then tests ADT;
   - opens the Windows password dialog if there is no valid password, and stores the password in the Windows
     Credential Manager;
   - reads the package list into **`packages.sappkg`** and opens it.

   The form shows ✓ / … / ✗ for each step. Errors explain what to do (URL and port, VPN, certificate, user or
   password, ADT not active, authorizations, single sign-on), and the details go to **`zray.log`**.
4. **Connect packages**: press `ctrl-.` on a package line in `packages.sappkg` and choose *connect package*. Or create
   a folder named after the package and answer the prompt.

`ctrl-.` in `zray.start` offers: *connect now*, *log in to SAP again*, and opening the guide, the AI instructions,
the log or the catalog.

**Each folder keeps its own system.** Once a folder is connected, it uses the system in `.zray/mirror.json`,
regardless of Zed's settings. Zed's `sap` settings only provide the system for a folder that doesn't have one yet, or
the properties file to use for that same system. This lets you keep folders for different systems open side by side,
each in its own window. A folder with connected packages refuses a different system: create a new folder instead.

Once connected, `ctrl-.` offers:

- **Zray: refresh from SAP**: fetches only what changed. **Refresh package X from SAP** does the same for the current
  file's package.
- **Zray: push this file to SAP** / **push all local changes to SAP**: writes your saved changes to SAP, then
  activates them. A prompt lists your transport requests as buttons, with the one you used last first. Changes made in
  SAP since your last pull are not overwritten unless you confirm. Activation errors are shown with file and line.
- **Zray: log in to SAP** opens the Windows credential dialog; **test the SAP connection** checks the connection.
- **Zray: analyze the whole package** runs every rule, including those that span objects (calls between programs,
  LUW, unused objects …). Progress appears in the status bar (cancellable); the cross-object findings appear as
  diagnostics, along with a report that links to a printable executive report.

The terminal equivalents are `zray sap login`, `zray sap packages --out <folder>` and
`zray sap pull <PACKAGE> --out <folder>`, with `--url --client --user` or `--system <file.properties>`. A bare host
means `http://host:8000`.

## The whole system: `analysis.sapanalysis`

In a connected folder, press `ctrl-.` in `zray.start` or on the header of `packages.sappkg` and choose *set up a full
system analysis*. This writes `analysis.sapanalysis` (six steps, one per line) and `analysis.json` (your choices).
Press `ctrl-.` on a step to run it. The steps read every customer object (sources, DDIC, CDS, messages, texts),
what those objects use from SAP, the where-used lists, transports and logs, the configuration tables and the usage
data, and then analyze it all. Each line shows its state: ● done, ○ not run (with an estimate), ◐ stopped (can be
resumed), ✗ failed, ⟳ out of date.

Zray asks before it reads anything: once per system, how to read (with the fetcher classes `ZCL_FETCHER_*` in `$TMP`,
or with SQL and nothing installed); once per run, whether to read; and on a production system, before every step
("Read PRODUCTION", always with SQL). For details, see section 3 of the folder's `README.md`.

## While you edit

- **Diagnostics** as you type: parse errors and the per-file rules, with the whole program as context (TOP include,
  other includes, class sections) and the folder's DDIC (`indexed-ddic.json`).
- **Code actions** (`ctrl-.`): the verified fixes for the statement under the cursor (SAFE fixes first), and
  "Apply all safe fixes in this file".
- **Go to definition** (`F12`), including members (`lo->attr`, `zcl_x=>const`, `ls_x-comp`); **hover** (declarations,
  DDIC fields and tables, and the keyword documentation for the system's release); **outline**; **workspace symbols**
  (`ctrl-t`); **completion** (keywords, names from the program, and members after `->`, `=>`, `~` and `-`).

## CDS

CDS sources (`.ddls`, `.ddlx`, `.dcls`, `.srvd`; abapGit `.asddls` …) open as **ABAP CDS**, with:

- highlighting and outline;
- the CDS checks as you type (access control, performance, unused associations, annotations, keys of to-one
  associations …; "unused view" needs the package analysis), and a fix for unused associations;
- go to definition and hover on data sources, associations and fields (through other views down to the DDIC);
- completion of fields after `.`, annotations after `@`, and views and tables after `from`, `join` and `to`.

In ABAP, hovering over a CDS view in a `SELECT` shows its elements.

## Behavior definitions (RAP)

Behavior definitions (`.bdef`, abapGit `.bdef.asbdef`) open as **ABAP BDEF**, with:

- highlighting and outline (entities, actions, determinations, validations, associations, fields);
- the RAP checks as you type (no authorization master, a draft lock master without a total etag, an unmanaged saver
  without SAVE / CLEANUP);
- go to definition and hover on the entities (their CDS views), their elements and associations, the persistent,
  draft and mapping tables and their fields (DDIC), and the behavior pool class;
- completion of the entity's elements and of BDEF keywords.

## AI assistants (MCP)

The same binary is also an MCP server: `zray mcp <folder>`. Zed's agent panel, Claude Code, Cursor and other AI
clients can then answer from the analysis: objects, routines, source, code search, findings and their rules, DDIC,
dependencies (where-used, call graph, table usage, related objects, influenced by), and a syntax check of the code
they write. It is read-only: nothing in SAP or in your files is changed.

**In Zed, it's built in:** the extension provides the MCP server **zray**. Open the agent panel → settings → the
server list, and turn on *zray*. Settings (`context_servers.zray.settings`):

- `folder`: defaults to the project the Zray language server opened last (open an ABAP file first);
- `tools`: `all`, or a selection such as `core,impact`; add `actions` for analyze and pull;
- `allowPush`: together with `actions`, allows pushing to SAP (the plan is shown first).

**Inline assistant: `@aic`.** Zed's inline assistant can't call tools (MCP), but it does take files as context. Put
the cursor on a statement, press `ctrl-.` and choose *AI context of this statement (@aic for the inline assistant)*.
Zray writes `.zray/aic.md` and tells you when it's ready. The file contains the statement, its routine and signature,
every name it uses resolved (variables with their types, DDIC fields, called methods), the findings with their
fixes, the verified fixes as before / after, the callers, and the source with the statement marked. Then open the
inline assistant (`ctrl-enter`), type **`@aic`** and ask ("fix the finding", "explain this", "rewrite with NEW").
Save the file first: the context is built from the saved source. From a terminal: `zray aic <file> --line N`.

**Other clients:** press `ctrl-.` and choose *use this folder in other AI assistants (MCP setup …)*. Zray writes
`.zray/mcp-setup.md` with the settings below, already filled in for this folder and this binary, and opens it.

Zed, configured by hand (not needed with the extension; in `settings.json`):

```json
"context_servers": {
  "zray": {
    "command": { "path": "C:/Users/<you>/AppData/Local/zray/zray.exe", "args": ["mcp", "C:/dev"] }
  }
}
```

Claude Code: `claude mcp add zray -- C:/Users/<you>/AppData/Local/zray/zray.exe mcp C:/dev`

`--tools=core` offers fewer tools, which uses less of the model's context. `--tools=all,actions` adds `analyze` and
`pull`; with `--allow-push`, it also adds `push`, which shows the plan first and pushes only after you confirm, into
the transport request you name. The cross-object findings are those from the last "analyze the whole package"; for
files changed since then, the per-file findings are computed on each call.

## Project configuration: `zray.json`

Every SAP folder (a folder connected to SAP) gets three files, written when they are missing:

- **`zray.json`**: this folder's rules, fixes, pretty print and analysis settings. It is shared by the team (commit
  it) and read by the editor, the MCP server and `zray aic`. Zed completes and validates it as you type (with the
  schema `.zray/zray.schema.json`). A key you change overrides Zed's `lsp.zray` setting; a key left at its default
  leaves Zed's setting in effect. Changes apply immediately. **To reset the file, delete it**: it is recreated with
  the defaults.
- **`ZRAY.md`**: a guide for developers (the actions, `@aic`, the MCP server, every key). Edit it as you like; it is
  only rewritten if you delete it.

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
| `rules.minSeverity` | hides findings below this severity (`info` … `critical`) |
| `rules.disabled` / `rules.enabled` | rules to hide / rules to show even though they are off by default (`CORE-006`) |
| `rules.severity` | overrides a rule's severity in this project |
| `rules.exclude` | files that get no findings: patterns matched against the path or the file name |
| `fixes.maxSafety` | the least safe fix that is offered: `safe`, `check`, `manual` |
| `fixes.never` | fixes that are never offered, nor applied by "apply all safe fixes" |
| `fixes.settings` | per-fix options, as `<FIX-ID>.<name>` |
| `prettyPrint.keywordCase` | `auto`, `upper`, `lower`: the case of keywords in completions and fixes |
| `analysis.release` | the ABAP release of the target system; `null` uses the mirror's |
| `analysis.afterPull` | `first`, `always`, `never`: whether to analyze the package after a pull |

### Production systems

Set `production = yes` in `zray.start`, or add `role=production` to the system's properties file
(`~/.abap-analysis/<system>.properties`; the folder's system is matched by its URL and client). Then:

- a push from the editor asks for confirmation again ("Push to PRODUCTION"), and `zray sap push` needs
  `--production`;
- the MCP server tells the AI that the system is production, and refuses `push`;
- the status bar shows **⛔ PRODUCTION SAP system · PRD client 100 (url)** for as long as the folder is open. (This is
  a language server progress indicator that never ends. Zed only applies themes from the user settings, so a folder
  can't change its own colors.)

The other roles (`quality`, `development`, `sandbox`) have no effect yet.

## Languages

Zray speaks **English** (`en`), **German** (`de`), **French** (`fr`), **Spanish** (`es`), **Japanese** (`ja`),
**Mandarin** (`zh`, simplified Chinese) and **Hebrew** (`he`). The language applies to the `zray.start` form, its
status and its actions, the folder's `README.md`, and the findings and parser messages in Problems and on hover.

- **Per folder:** set `language = de` in `zray.start` and save. The folder keeps the choice (`.zray/user.json`), the
  form and `README.md` are rewritten in that language, and every open file is analyzed again. The form's `language`
  line lists the supported languages.
- **For all folders:** use the `language` setting below (`"de"`). A folder's own choice takes precedence.
- **Hebrew displays poorly:** Zed has no right-to-left support and shows Hebrew reversed. It remains available, and
  the form and a message (in English) point this out.

When the catalog has no exact translation for a finding, the rule's title is shown in your language above the English
message. `AGENTS.md` stays in English (AI assistants follow it best) and tells the assistant to answer in your
language. The form's keys (`url`, `client` …), ABAP keywords, file names and SAP terms are not translated.

## Settings (`initialization_options` or `settings` of `lsp.zray`)

| Key | Default | Meaning |
|---|---|---|
| `release` | the mirror's, else 750 | the ABAP release whose syntax fixes generate (`758` or `"7.58"`) |
| `minSeverity` | `info` | hides findings below `info` / `low` / `medium` / `high` / `critical` |
| `disabledRules` | `[]` | rule IDs to hide |
| `enabledRules` | `[]` | rule IDs to show even though they are off by default (e.g. `CORE-006`) |
| `reference` | `ABAP_REFERENCE` | the SAP reference data folder (`<release>_JSON`) |
| `keywordCase` | `auto` | the case of completed keywords: `auto` (as typed), `upper`, `lower` |
| `cache` / `cacheDir` | `true` / `%LOCALAPPDATA%\zray\lsp\<hash>` | the facts cache (never stored inside the project) |
| `sap` | — | `{ "url", "client", "user" }` or `{ "system": "<file.properties>" }`: the system for a folder that has none yet (the folder's own `.zray/mirror.json` takes precedence), and the properties file used for that same system |
| `analyzeAfterPull` | `first` | analyzes the whole package in the background after connecting it (`first`), also after every refresh that changed files (`always`), or `never`. Starting a command stops it, and it runs again afterwards |
| `language` | `en` | the language of Zray's texts and findings: `en`, `de`, `fr`, `es`, `ja`, `zh` or `he` (shown reversed in Zed); `language` in a folder's `zray.start` takes precedence |
| `manifestUrl` | the latest release of this repository | (`settings` only) the release manifest the extension downloads `zray` from when it is neither set in `binary.path` nor on the `PATH`; `""` turns downloading off |

Severity mapping: critical → error, high / medium → warning, low → information, info → hint.

## License and grammars

The extension is MIT-licensed (`LICENSE`); the language server it starts is proprietary Zray software.
Highlighting, brackets, indentation and outline come from Zray's own grammars in `tree-sitter-zray` (MIT): `abap`,
`abap_cds` and `abap_bdef`. Their queries are in `languages/*/*.scm`.
