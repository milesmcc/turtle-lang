((lambda (x) x) 'g)

(cond ('t 'hi))

(label name t)

name

(label subst 
    (lambda (x y z)
        (cond 
            ((atom z)
                (cond 
                    ((eq z y) x)
                    ('t z)
                )
            )
            ('t (cons (subst x y (car z))
            (subst x y (cdr z))))
        )
    )
)

(subst 'm 'b '(a b (a b c) d))