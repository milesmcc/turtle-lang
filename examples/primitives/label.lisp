(cond ('t 'hi))

(label name t)

(label temp (lambda (x) (label name x)))

(label deep (lambda (x) (temp x)))

(deep 'yolo)

name


(label subst 
    (lambda (x y z)
        (cond 
            (
                (atom z)
                (cond 
                    ((eq z y) x)
                    ('t z)
                )
            )
            (
                't
                (cons (subst x y (car z)) (subst x y (cdr z)))
            )
        )
    )
)

(subst 'a 'b '(b))

// (subst nil nil '(a b (a b c) d))