((lambda (x) (cons x '(b))) 'a)

((lambda (x y) (cons x (cdr y)))
    'z
    '(a b c))

((lambda (f) (f '(b c)))
    '(lambda (x) (cons 'a x)))
