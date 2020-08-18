(import "@prelude")

(func contains 
    (key map) 
    (gt 0 
        (length 
            (filter 
                (lambda '
                    (k) '
                    (eq 
                        (first k) 
                        key)) map))))
(func insert 
    (kvpair map) 
    (cons kvpair 
        (filter 
            (lambda '
                (k) '
                (not 
                    (eq 
                        (first k) 
                        (first kvpair)))) map)))
(metafunc insert! ($kvpair $map) (let $map (insert ,$kvpair ,$map)))
(let 'remove. (lambda 
    '(key map) 
        '(filter 
            (lambda '
                (k) '
                (not 
                    (eq 
                        (first k) 
                        key))) map)))
(metafunc remove! ($key $map) (let $map (remove. ,$key ,$map)))
(export 'remove remove.)
(func extract 
    (key map) 
    (second 
        (first 
            (filter 
                (lambda '
                    (k) '
                    (eq 
                        (first k) 
                        key)) map))))