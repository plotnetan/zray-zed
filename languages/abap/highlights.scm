[(comment) (line_comment)] @comment
(pragma) @preproc
(string) @string
(string_template) @string
(number) @number
(keyword) @keyword
(field_symbol) @variable
(selector) @operator
(operator) @operator
(punctuation) @punctuation.delimiter
"." @punctuation.delimiter
["(" ")" "[" "]"] @punctuation.bracket

; the self reference and the system fields
((identifier) @variable.special
  (#match? @variable.special "^(?i)(me|super|sy|syst)$"))
((identifier) @constant.builtin
  (#match? @constant.builtin "^(?i)(abap_true|abap_false|abap_undefined|space)$"))

; calls: the method / function name (the last segment)
(call function: (identifier) @function)
(call function: (member_access (identifier) @function .))

; the names of units
(class_statement name: (_) @type)
(interface_statement name: (_) @type)
(method_statement name: (_) @function)
(methods_statement name: (_) @function)
(form_statement name: (_) @function)
(function_statement name: (_) @function)
(module_statement name: (_) @function)
(program_statement name: (_) @type)

; parameter declarations (METHODS, CLASS-METHODS, FORM): a name before TYPE / LIKE, `!name`, `VALUE( name )`,
; `REFERENCE( name )`
(methods_statement (identifier) @variable.parameter . (keyword) @_t (#match? @_t "^(?i)(type|like)$"))
(form_statement (identifier) @variable.parameter . (keyword) @_t (#match? @_t "^(?i)(type|like)$"))
(methods_statement (identifier) @variable.parameter (#match? @variable.parameter "^!"))
(methods_statement
  (keyword) @_v (#match? @_v "^(?i)(value|reference)$")
  .
  (parenthesized (identifier) @variable.parameter))
