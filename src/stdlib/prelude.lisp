(label 'set label)

(label 'setq (macro (identifier value) (label identifier ,value)))

(label 'func (macro args (label (car args) (lambda (car (cdr args)) (cdr (cdr args))))))

;; Basic math operators  
(setq + sum)
(setq * prod)
(setq - (lambda (a, b) (+ a (* -1 b))))
(setq / (lambda (a, b) (* a (pow b -1))))
(setq % modulo)

(setq ++ (macro (a) (set a (+ ,a 1)) ,a))
(setq -- (macro (a) (set a (- ,a 1)) ,a))

(setq zen "The Zen of Turtle (to be written...)")