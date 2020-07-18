(cond ((eq 'a 'b) 'first)
       ((atom '(a b)) 'second)
       ((eq 't 't) 'third))