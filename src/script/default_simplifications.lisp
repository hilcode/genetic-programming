; Default simplification rules bundled with the GP engine.
; Applied automatically before any user-defined rules.
; Rules for operators that do not exist in a domain are silently ignored —
; a pattern can only match a node that is already in the tree.

; ── Boolean constants ─────────────────────────────────────────────────────────

(simplification "not-true"    '(NOT TRUE)   'FALSE)
(simplification "not-false"   '(NOT FALSE)  'TRUE)
(simplification "not-not"     '(NOT (NOT ?x))  '?x)

(simplification "and-false-l" '(AND FALSE ?x)  'FALSE)
(simplification "and-false-r" '(AND ?x FALSE)  'FALSE)
(simplification "and-true-l"  '(AND TRUE ?x)   '?x)
(simplification "and-true-r"  '(AND ?x TRUE)   '?x)

(simplification "or-true-l"   '(OR TRUE ?x)    'TRUE)
(simplification "or-true-r"   '(OR ?x TRUE)    'TRUE)
(simplification "or-false-l"  '(OR FALSE ?x)   '?x)
(simplification "or-false-r"  '(OR ?x FALSE)   '?x)

(simplification "xor-same"    '(XOR ?x ?x)     'FALSE)
(simplification "xor-false-l" '(XOR FALSE ?x)  '?x)
(simplification "xor-false-r" '(XOR ?x FALSE)  '?x)

; ── Conditional ───────────────────────────────────────────────────────────────

(simplification "if-true"     '(IF TRUE ?then ?else)   '?then)
(simplification "if-false"    '(IF FALSE ?then ?else)  '?else)
(simplification "if-same"     '(IF ?cond ?x ?x)        '?x)

; ── Arithmetic constants ──────────────────────────────────────────────────────

(simplification "add-zero-l"  '(+ 0 ?x)   '?x)
(simplification "add-zero-r"  '(+ ?x 0)   '?x)
(simplification "sub-zero"    '(- ?x 0)   '?x)
(simplification "mul-zero-l"  '(* 0 ?x)   '0)
(simplification "mul-zero-r"  '(* ?x 0)   '0)
(simplification "mul-one-l"   '(* 1 ?x)   '?x)
(simplification "mul-one-r"   '(* ?x 1)   '?x)
