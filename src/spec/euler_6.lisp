;; From https://projecteuler.net/problem=6

(import "@prelude")
(import "@math")

(letq sum-of-squares 
    (apply sum
        (map square (range 100))))

(letq square-of-sum
    (square (apply sum (range 100))))

(letq answer (- square-of-sum sum-of-squares))

(assert (eq answer 25164150))