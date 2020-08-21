;; From https://projecteuler.net/problem=5
;;
;; 2520 is the smallest number that can be divided by each of the numbers from 1 to 10 without any remainder.
;;
;; What is the smallest positive number that is evenly divisible by all of the numbers from 1 to 20?

(import "@prelude")
(import "@math")

(letq factors '(20 19 18 17 16 15 14 13 12 11))

(letq interval (* 20 (apply * (filter is-prime factors))))

(letq i interval)
(while
    (not (apply and (map (lambda '(x) '(eq (modulo i x) 0)) factors)))
    (letq i (+ i interval)))

(assert (eq i 232792560))
