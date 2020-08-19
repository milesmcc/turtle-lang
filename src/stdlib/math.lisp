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

;; Primes
(func next-prime 
    (primes) 
    (do 
        (let 'n 
            (last primes)) 
        (while ,
            (cons or 
                (map 
                    (lambda '
                        (divisor) '
                        (eq 
                            (modulo n divisor) 0)) primes)) 
            (++ n)) n))
(func primes 
    (n) 
    (cond 
        (
            (eq 0 n) 
            ()) 
        (
            (eq 1 n) '
            (2)) 
        ('t 
            (do 
                (let 'previous 
                    (primes 
                        (+ n -1))) 
                (append previous 
                    (list 
                        (next-prime previous)))))))
(func is-prime 
    (n) 
    (do 
        (letq is-composite 
            ()) 
        (letq p 2) 
        (letq tried 
            ()) 
        (while 
            (and 
                (not is-composite) 
                (strictly-increasing p n)) 
            (? 
                (eq 
                    (modulo n p) 0) 
                (letq is-composite 't) 
                (do 
                    (push! tried p) 
                    (letq p 
                        (next-prime tried))))) (not is-composite)))
(func prime-factorization 
    (n) 
    (do 
        (letq factors ()) 
        (letq curr n) 
        (while 
            (not (eq ,(append '(prod) factors) n))
            (do
                (let 'trying 2)
                (while
                    (not (eq (modulo curr trying) 0))
                    (++ trying))
                (push! factors trying)
                (let 'curr (/ curr trying))))
     factors))
