;; From https://projecteuler.net/problem=6

(import "@prelude")
(import "@math")

(letq sum-of-squares ,(cons sum (map (lambda '(x) '(exp x 2)) (range 100))))
(letq square-of-sum (exp ,(cons sum (range 100)) 2))
(letq answer (- square-of-sum sum-of-squares))

(disp answer)

(assert (eq answer 25164150))
