;; Close-to-primitive operators
(export 'exportq (macro '(identifier value) '(export identifier ,value)))
(export 'letq (macro '(identifier value) '(let identifier ,value)))

(export 'set export)
(export 'setq exportq)

;; Helpful list operators
(export 'head car)
(export 'tail cdr)
(export 'first (lambda '(x) '(head x)))
(export 'second (lambda '(x) '(head (tail x))))
(export 'third (lambda '(x) '(head (tail (tail x)))))
(export 'fourth (lambda '(x) '(head (tail (tail (tail x))))))

;; Macros
(export 'metafunc (macro 'args '(export (first args) (macro (second args) (first (tail (tail args)))))))
(metafunc func args (export (first args) (lambda (second args) (first (tail (tail args))))))

;; Operation shorthand
(metafunc do something ,(cons lambda (cons () something)))

;; Assertion and testing
(func assert (expr) (cond (expr expr) ('t (throw :assertion-failed-exp))))

;; Math constants
(export 'pi 3.14159265358979323846)
(export 'e 2.71828182845904523536)

;; Basic math operators  
(export '+ sum)
(export '* prod)
(func - (a b) (+ a (* -1 b)))
(func / (a b) (* a (exp b -1)))
(export '% modulo)
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