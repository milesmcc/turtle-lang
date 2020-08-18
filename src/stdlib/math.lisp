(import "@prelude")

;; Trignometry
(let 'sinseries '
    (lambda '
        (x) '
        (+ x 
            (/ 
                (exp x 3) -6) 
            (/ 
                (exp x 5) 120) 
            (/ 
                (exp x 7) -5040) 
            (/ 
                (exp x 9) 362880) 
            (/ 
                (exp x 11) -39916800) 
            (/ 
                (exp x 13) 6227020800) 
            (/ 
                (exp x 15) 
                (* -1 
                    (fac 15))))))
(export 'sin '
    (lambda '
        (x) '
        (* -1 
            (sinseries 
                (- 
                    (modulo x tau) pi)))))
(export 'cos '
    (lambda '
        (x) '
        (sin 
            (+ x 
                (/ pi 2)))))
(export 'tan '
    (lambda '
        (x) '
        (/ 
            (sin x) 
            (cos x))))

;; Sequences
(func fibonacci
    (n) 
    (cond 
        (
            (eq n 0) '
            (0)) 
        (
            (eq n 1) '
            (0 1)) 
        ('t 
            (do 
                (letq sequence 
                    (fibonacci 
                        (+ n -1))) 
                (append sequence 
                    (list 
                        (+ 
                            (nth 
                                (+ n -2) sequence) 
                            (nth 
                                (+ n -1) sequence))))))))