;; Minimal test domain: just add two numbers to reach a target

(operator "+" "Num" (list "Num" "Num")
  (lambda (args ctx)
    (+ (nth args 0) (nth args 1))))

(terminal "TARGET" "Num"
  (lambda (ctx) (get ctx "target")))

(terminal "ONE" "Num"
  (lambda (ctx) 1))

(fitness
  (lambda (tree)
    (let ((result (eval-tree tree (list (list "target" 5)))))
      ;; Fitness is negative absolute error (higher is better)
      (- 0.0 (abs (- result 5.0))))))
