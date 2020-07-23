;; Close-to-primitive operators
(label 'set label)
(label 'setq (macro '(identifier value) '(label identifier ,value)))

;; Helpful list operators
(setq head car)
(setq tail cdr)
(setq first (lambda '(x) '(head x)))
(setq second (lambda '(x) '(head (tail x))))
(setq third (lambda '(x) '(head (tail (tail x)))))
(setq fourth (lambda '(x) '(head (tail (tail (tail x))))))

;; Macros
(label 'metafunc (macro 'args '(label (first args) (macro (second args) (first (tail (tail args)))))))
(metafunc func args (label (first args) (lambda (second args) (first (tail (tail args))))))

;; Operation shorthand
(metafunc do something ,(cons lambda (cons () something)))

;; Assertion and testing
(func assert (expr) (cond (expr expr) ('t (throw :assertion-failed-exp))))

;; Math constants
(setq pi 3.14159265358979323846)
(setq e 2.71828182845904523536)

;; Basic math operators  
(setq + sum)
(setq * prod)
(func - (a b) (+ a (* -1 b)))
(func / (a b) (* a (exp b -1)))
(setq % modulo)
(metafunc ++ (a) (set a (+ ,a 1)))
(metafunc -- (a) (set a (+ ,a -1)))
(metafunc increasing elems ,(cons ge elems))
(metafunc strictly-increasing elems ,(cons gt elems))
;; implement increasing and strictly increasing using the reverse operator

;; Trigonometry
;; TODO

;; Boolean operators
(func not (val) (cond (val ()) ('t 't)))
(func and vals (cond ((not vals) 't) ((head vals) (and (tail vals))) ('t ())))
(func or vals (cond ((not vals) ()) ((head vals) 't) ('t (or (tail vals)))))

;; Fun
(setq zen "The Zen of Turtle (to be written...)")