(import "@prelude")

(assert (eq "smoke" "smoke"))
(assert (not (eq "smoke" "smoketest")))

(assert (eq 5 (+ 2 3)))

(assert (or () () 1 ()))
(assert (not (or () () () ())))
