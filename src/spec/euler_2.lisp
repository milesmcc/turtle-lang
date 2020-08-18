;; From https://projecteuler.net/problem=2
;;
;; Each new term in the Fibonacci sequence is generated by adding the previous two terms. By starting with 1 and 2, the first 10 terms will be:
;;
;; 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, ...
;;
;; By considering the terms in the Fibonacci sequence whose values do not exceed four million, find the sum of the even-valued terms.

(import "@prelude")

(let 'p2 0)
(let 'p1 1)

(func fib 
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
                    (fib 
                        (+ n -1))) 
                (append sequence 
                    (list 
                        (+ 
                            (nth 
                                (+ n -2) sequence) 
                            (nth 
                                (+ n -1) sequence))))))))

(let 'answer ,
    (append '
        (sum) 
        (filter 
            (lambda '
                (k) '
                (eq 
                    (modulo k 2) 0)) 
            (fib 33))))
(assert 
    (eq answer 4613732))
