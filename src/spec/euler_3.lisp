;; From https://projecteuler.net/problem=3
;;
;; The prime factors of 13195 are 5, 7, 13 and 29.
;;
;; What is the largest prime factor of the number 600851475143 ?

(import "@prelude")
(import "@math" :math)

(let 'n 600851475143)

(assert (eq (last (math::prime-factorization n)) 6857))