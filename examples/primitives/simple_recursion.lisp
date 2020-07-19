(car '(a b c))

(label recurse 
    (lambda (x) 
        (cond ((atom x) x)
              ('t (recurse (car x))))))

(recurse '(((x))))

(recurse (cons 'b '(y z)))