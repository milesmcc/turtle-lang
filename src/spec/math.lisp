(import "@prelude")

;; Greater than
(assert (gt 0 1 2 3 4))
(assert (not (gt 3 2 1)))
(assert (gt -5 -4.0 -3 -2 1 2 3 4 500))

;; Greater than or equal to
(assert (ge -1 -0 0 1 1 2))
(assert (not (ge -5 -5 -4 -5)))

;; Sums
(assert (eq (sum 5 4 3 2 1) 15))
(assert (eq (sum 5 4 3 2 -1) 13))

;; Modulo
(assert (eq (modulo 5 3) 2))
(assert (eq (modulo 5 -3) 2))

;; Products
(assert (eq (prod 5 4 3 2 1) 120))
(assert (eq (prod 5 4 3 2 -1) -120))

;; Math shorthands
(assert (eq (+ 5 4 3) 12))
(assert (eq (- 5 4) 1))
(assert (eq (- 5 -5) 10))
(assert (eq (* 5 4 3 2 1) 120))
(assert (eq (/ 1 10) 0.1))

;; Increasing & decreasing
(assert (increasing 5))
(assert (increasing 1 2 3 4 5))
(assert (not (increasing 1 2 8 3 4 5)))
(assert (increasing 1 2 3 3 4 5))

(assert (strictly-increasing 5))
(assert (strictly-increasing 1 2 3 4 5))
(assert (not (strictly-increasing 1 2 8 3 4 5)))
(assert (not (strictly-increasing 1 2 3 3 4 5)))

(assert (decreasing 5))
(assert (decreasing 5 4 3 2 1))
(assert (not (decreasing 5 4 3 2 3 4 5)))
(assert (decreasing))

(assert (strictly-decreasing 5))
(assert (strictly-decreasing 5 4 3 2 1))
(assert (not (strictly-decreasing 1 2 8 3 4 5)))
(assert (not (strictly-decreasing 5 4 4 3 2)))
