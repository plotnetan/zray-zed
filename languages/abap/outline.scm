(class_statement (keyword) @context name: (_) @name) @item
(interface_statement (keyword) @context name: (_) @name) @item
(method_statement (keyword) @context name: (_) @name) @item
(methods_statement (keyword) @context name: (_) @name) @item
(form_statement (keyword) @context name: (_) @name) @item
(function_statement (keyword) @context name: (_) @name) @item
(module_statement (keyword) @context name: (_) @name) @item
(program_statement (keyword) @context name: (_) @name) @item
(event_statement (keyword) @name) @item
(statement
  .
  (keyword) @context
  (#match? @context "^(?i)at$")
  .
  (keyword) @name
  (#match? @name "^(?i)(selection-screen|line-selection|user-command)$")) @item
