(comment) @comment
(string) @string
(number) @number
(keyword) @keyword
(self) @variable.special
(association_name) @property
(operator) @operator
(punctuation) @punctuation.delimiter
["{" "}" "(" ")" "[" "]"] @punctuation.bracket
(behavior_definition name: (identifier) @type)
(member_definition name: (identifier) @function)
(association_definition name: (_) @property)
