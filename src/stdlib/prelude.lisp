;; Close-to-primitive operators
(label 'set label)
(label 'setq (macro '(identifier value) '(label identifier ,value)))

(setq head car)
(setq tail cdr)

(setq first (lambda '(x) '(head x)))
(setq second (lambda '(x) '(head (tail x))))
(setq third (lambda '(x) '(head (tail (tail x)))))
(setq fourth (lambda '(x) '(head (tail (tail (tail x))))))

(label 'metafunc (macro 'args '(label (first args) (macro (second args) (first (tail (tail args)))))))
(metafunc func args (label (first args) (lambda (second args) (first (tail (tail args))))))

;; Basic math operators  
(setq + sum)
(setq * prod)
(func - (a b) (+ a (* -1 b)))
(func / (a b) (* a (pow b -1)))
(setq % modulo)

(metafunc ++ (a) (set a (+ ,a 1)))
(metafunc -- (a) (set a (+ ,a -1)))

(setq zen "The Zen of Turtle (to be written...)")