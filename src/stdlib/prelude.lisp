(label 'set label)
(label 'setq (macro '(identifier value) '(label identifier ,value)))

(setq head car)
(setq tail cdr)

(setq first (lambda '(x) '(head x)))
(setq second (lambda '(x) '(head (tail x))))
(setq third (lambda '(x) '(head (tail (tail x)))))
(setq fourth (lambda '(x) '(head (tail (tail (tail x))))))

(setq func (macro 'args '(setq '(first args) (lambda '(second args) (tail (tail args))))))

;; Basic math operators  
(setq + sum)
(setq * prod)
(func - (a b) (+ a (* -1 b)))
(setq / (lambda (a b) (* a (pow b -1))))
(setq % modulo)

(setq ++ (macro (a) (set a (+ ,a 1)) ,a))
(setq -- (macro (a) (set a (- ,a 1)) ,a))

(setq zen "The Zen of Turtle (to be written...)")