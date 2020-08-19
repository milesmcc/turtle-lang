;; From https://projecteuler.net/problem=3
;;
;; The prime factors of 13195 are 5, 7, 13 and 29.
;;
;; What is the largest prime factor of the number 600851475143 ?

(import "@prelude")
(import "@math" :math)

(let 'n 600851475143)
(let 'factors ())
(let 'curr n)

(while 
    (not (eq ,(append '(prod) factors) n))
    (do
        (let 'trying 2)
        (while
            (not (eq (modulo curr trying) 0))
            (++ trying))
        (push! factors trying)
        (let 'curr (/ curr trying))))

(assert (eq (last factors) 6857))