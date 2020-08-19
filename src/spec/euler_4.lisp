;; From https://projecteuler.net/problem=4
;;
;; A palindromic number reads the same both ways. The largest palindrome made from the product of two 2-digit numbers is 9009 = 91 × 99.
;;
;; Find the largest palindrome made from the product of two 3-digit numbers.

(import "@prelude")
(import "@math" :math)

(func digits (n)
    (do
        (letq curr n)
        (letq digi ())
        (letq order 0)
        (while (gt 0 curr)
            (do
                (letq last-digit (modulo curr 10))
                (letq digi (cons last-digit digi))
                (letq curr (/ (- curr last-digit) 10))))
        digi))

(func is-palindrome (n)
    (do
        (let 'ns (digits n))
        (equiv ns (reverse ns))))

(letq a 999)
(letq b 999)
(letq palindromes ())

(while
    (and (ge 900 a))
    (do
        (if (is-palindrome (* a b)) (do (push! palindromes (* a b))))
        (cond
            ((gt a b) (-- b))
            ('t (do
                    (-- a)
                    (letq b 999))))))

(assert (eq (last (sort palindromes)) 906609))
