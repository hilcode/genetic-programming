; Flag problem: evolve an expression that returns 42 when flag=true and 123 when flag=false.
; The simplest solution is (IF FLAG 42 123).
;
; Context association list keys:
;   "target"  — Num, always 100
;   "flag"    — Bool

; ── Numeric operators ─────────────────────────────────────────────────────────

(operator "+" "Num" (list "Num" "Num")
  (lambda (args ctx) (+ (nth args 0) (nth args 1))))

(operator "-" "Num" (list "Num" "Num")
  (lambda (args ctx) (- (nth args 0) (nth args 1))))

(operator "*" "Num" (list "Num" "Num")
  (lambda (args ctx) (* (nth args 0) (nth args 1))))

(operator "IF" "Num" (list "Bool" "Num" "Num")
  (lambda (args ctx) (if (nth args 0) (nth args 1) (nth args 2))))

; ── Boolean operators ─────────────────────────────────────────────────────────

(operator "NOT" "Bool" (list "Bool")
  (lambda (args ctx) (not (nth args 0))))

(operator "AND" "Bool" (list "Bool" "Bool")
  (lambda (args ctx) (and (nth args 0) (nth args 1))))

(operator "OR" "Bool" (list "Bool" "Bool")
  (lambda (args ctx) (or (nth args 0) (nth args 1))))

(operator "XOR" "Bool" (list "Bool" "Bool")
  (lambda (args ctx) (not (= (nth args 0) (nth args 1)))))

; ── Numeric terminals ─────────────────────────────────────────────────────────

(terminal "TARGET" "Num" (lambda (ctx) (get ctx "target")))

(terminal "-10" "Num" (lambda (ctx) -10))
(terminal "-9"  "Num" (lambda (ctx) -9))
(terminal "-8"  "Num" (lambda (ctx) -8))
(terminal "-7"  "Num" (lambda (ctx) -7))
(terminal "-6"  "Num" (lambda (ctx) -6))
(terminal "-5"  "Num" (lambda (ctx) -5))
(terminal "-4"  "Num" (lambda (ctx) -4))
(terminal "-3"  "Num" (lambda (ctx) -3))
(terminal "-2"  "Num" (lambda (ctx) -2))
(terminal "-1"  "Num" (lambda (ctx) -1))
(terminal "0"   "Num" (lambda (ctx) 0))
(terminal "1"   "Num" (lambda (ctx) 1))
(terminal "2"   "Num" (lambda (ctx) 2))
(terminal "3"   "Num" (lambda (ctx) 3))
(terminal "4"   "Num" (lambda (ctx) 4))
(terminal "5"   "Num" (lambda (ctx) 5))
(terminal "6"   "Num" (lambda (ctx) 6))
(terminal "7"   "Num" (lambda (ctx) 7))
(terminal "8"   "Num" (lambda (ctx) 8))
(terminal "9"   "Num" (lambda (ctx) 9))
(terminal "10"  "Num" (lambda (ctx) 10))

; ── Boolean terminals ─────────────────────────────────────────────────────────

(terminal "TRUE"  "Bool" (lambda (ctx) true))
(terminal "FALSE" "Bool" (lambda (ctx) false))
(terminal "FLAG"  "Bool" (lambda (ctx) (get ctx "flag")))

; ── Fitness ───────────────────────────────────────────────────────────────────

(fitness (lambda (node)
  (let ((true-result  (eval-tree node (list (list "target" 100) (list "flag" true))))
        (false-result (eval-tree node (list (list "target" 100) (list "flag" false)))))
    (let ((true-diff  (abs (- true-result  42)))
          (false-diff (abs (- false-result 123))))
      (* (/ 1.0 (+ 1.0 true-diff))
         (/ 1.0 (+ 1.0 false-diff)))))))
