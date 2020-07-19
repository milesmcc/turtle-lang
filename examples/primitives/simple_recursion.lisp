(car '(a b c))

(label recurse 
    (lambda (x) 
        (cond ((atom x) x)
              ('t (recurse (car x))))))

(recurse '(((a))))

(recurse (cons 'x '(y x)))

(= x '(a b x))