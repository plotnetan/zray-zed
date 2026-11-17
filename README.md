# ABAP for Zed

ABAP, CDS and RAP behavior definitions in [Zed](https://zed.dev): highlighting, brackets, indentation and outline.

This version has syntax support only. Diagnostics, verified fixes, navigation, the SAP connection (ADT) and the MCP
server for Zed's agent come with the Zray language server in a later version (branch `lsp`).

## Languages

| Language | Files |
| --- | --- |
| ABAP | `.abap` |
| ABAP CDS | data definitions, metadata extensions, access controls, service definitions: `.ddls`, `.ddlx`, `.dcls`, `.srvd`; abapGit `.asddls`, `.asddlxs`, `.asdcls`, `.srvdsrv` |
| ABAP BDEF | behavior definitions: `.bdef`; abapGit `.asbdef` |

The outline lists the program, classes, interfaces, methods, forms, function modules, dialog modules and events in
ABAP; the definitions (views, entities, metadata extensions, access controls, services) in CDS; the entities with
their actions, determinations, validations and associations in BDEF.

## Install

From Zed's extension registry: command palette → `zed: extensions` → search for **ABAP**.

As a development extension (needs [Rust](https://rustup.rs) and git): clone this repository, then command palette →
`zed: install dev extension` → the cloned folder.

## License and grammars

MIT (`LICENSE`). The grammars are Zray's own, [`tree-sitter-zray`](https://github.com/plotnetan/tree-sitter-zray)
(MIT): `abap`, `abap_cds`, `abap_bdef`; the editor queries are `languages/*/*.scm`.
