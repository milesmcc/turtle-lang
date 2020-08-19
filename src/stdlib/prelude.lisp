;; Close-to-primitive operators
(export 'exportq 
    (macro '
        (identifier value) '
        (export identifier ,value)))

(export 'letq 
    (macro '
        (identifier value) '
        (let identifier ,value)))

(export 'set export)
(export 'setq exportq)

;; Helpful list operators
(export 'drop 
    (lambda '
        (n xs) '
        (cond 
            (
                (gt n 1) xs) 
            ('t 
                (drop 
                    (sum n -1) 
                    (tail xs))))))
(export 'nth 
    (lambda '
        (n xs) '
        (car 
            (drop n xs))))
(export 'head car)
(export 'tail cdr)
(export 'first 
    (lambda '
        (x) '
        (nth 0 x)))
(export 'second 
    (lambda '
        (x) '
        (nth 1 x)))
(export 'third 
    (lambda '
        (x) '
        (nth 2 x)))
(export 'last 
    (lambda '
        (x) '
        (nth 
            (sum 
                (length x) -1) x)))
(export 'filter
    (lambda
        '(criteria lst)
        '(do
            (let 'matches ())
            (let 'n 0)
            (while
                (gt n (length lst))
                (do
                    (let 'val (nth n lst))
                    (cond
                        (
                            (criteria val)
                            (letq matches (append matches (list val))))
                        ('t ()))
                    (++ n)))
            matches)))
(export 'remove 
    (lambda '
        (n xs) '
        (cond 
            (
                (eq n 0) 
                (tail xs)) 
            ('t 
                (cons 
                    (first xs) 
                    (remove 
                        (sum n -1) 
                        (tail xs)))))))
(export 'reverse 
    (lambda '
        (xs) '
        (cond 
            (xs 
                (cons 
                    (last xs) 
                    (reverse 
                        (remove 
                            (sum 
                                (length xs) -1) xs)))) 
            ('t 
                ()))))

;; Macros
(export 'metafunc 
    (macro 'args '
        (export 
            (first args) 
            (macro 
                (second args) 
                (first 
                    (tail 
                        (tail args)))))))
(metafunc func args 
    (export 
        (first args) 
        (lambda 
            (second args) 
            (first 
                (tail 
                    (tail args))))))

;; Assertion and testing
(func assert 
    (expr) 
    (cond 
        (expr expr) 
        ('t 
            (throw :assertion-failed-exp))))

;; Math constants
(export 'pi 3.14159265358979323846)
(export 'tau 
    (prod 2 pi))
(export 'e 2.71828182845904523536)

;; Basic math operators  
(export '+ sum)
(export '* prod)
(func - 
    (a b) 
    (+ a 
        (* -1 b)))
(func / 
    (a b) 
    (* a 
        (exp b -1)))
(export '% modulo)
(metafunc ++ 
    (a) 
    (set a 
        (+ ,a 1)))
(metafunc -- 
    (a) 
    (set a 
        (+ ,a -1)))
(metafunc increasing elems ,
    (cons ge elems))
(metafunc strictly-increasing elems ,
    (cons gt elems))
(metafunc decreasing elems ,
    (cons ge 
        (reverse elems)))
(metafunc strictly-decreasing elems ,
    (cons gt 
        (reverse elems)))
(export 'fac '
    (lambda '
        (x) '
        (cond 
            (
                (ge x 0) 1) 
            ('t 
                (* x 
                    (fac 
                        (- x 1)))))))
(metafunc ++ 
    (arg) 
    (let arg 
        (+ ,arg 1)))
(metafunc -- 
    (arg) 
    (let arg 
        (+ ,arg -1)))

;; Boolean operators
(func not 
    (val) 
    (cond 
        (val 
            ()) 
        ('t 't)))
(func and vals 
    (cond 
        (
            (not vals) 't) 
        (
            (head vals) 
            ,(cons and 
                (tail vals))) 
        ('t 
            ())))
(func or vals 
    (cond 
        (
            (not vals) 
            ()) 
        (
            (head vals) 't) 
        ('t 
         ,
            (cons or 
                (tail vals)))))
(metafunc if 
    (val todo) 
    (cond 
        (,val ,todo) 
        ('t 
            ())))
(metafunc ? 
    (val if else) 
    (cond 
        (,val ,if) 
        ('t ,else)))

;; More list helpers
(func in 
    (lst val) 
    (gt 0 
        (length 
            (filter 
                (lambda '
                    (k) '
                    (eq k val)) lst))))
(func map 
    (function args) 
    (cond 
        (
            (eq args 
                ()) 
            ()) 
        ('t 
            (cons 
                (function 
                    (first args)) 
                (map function 
                    (tail args))))))
(func push 
    (lst val) 
    (append lst 
        (list val)))
(metafunc push! 
    (lst val) 
    (let lst 
        (push ,lst ,val)))
(func sort (lst)
    (cond
        (,(append '(ge) lst) lst)
        (
            't
            (do
                (let 'pivot (nth (floor (* (rand) (length lst))) lst))
                (let 'left (filter (lambda '(x) '(gt x pivot)) lst))
                (let 'right (filter (lambda '(x) '(ge pivot x)) lst))
                (append (sort left) (sort right))))))
(func range (n)
    (do
        (let 'nums ())
        (while
            (gt 0 n)
            (do
                (let 'nums (cons n nums))
                (-- n)))
        nums))

;; Fun
(setq zen "The Zen of Turtle (to be written...)")
