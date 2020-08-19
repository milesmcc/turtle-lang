;; From https://projecteuler.net/problem=1
;;
;; If we list all the natural numbers below 10 that are multiples of 3 or 5, we get 3, 5, 6 and 9. The sum of these multiples is 23.
;;
;; Find the sum of all the multiples of 3 or 5 below 1000.

(import "@prelude")

(let 'i 0)
(let 'sum 0)

(while (gt i 1000) 
    (do (if (in (list (modulo i 3) (modulo i 5)) 0)
            (let 'sum (+ sum i)))
        (++ i)))

(assert (eq sum 233168))
