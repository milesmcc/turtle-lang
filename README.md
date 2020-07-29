# turtle
A Lispy programming language that, minus a small shell, is written in itself. It's turtles all the way down. 

## TODO
- [x] Figure out why `(eq func func)` is false
- [x] Figure out namespaces
- [x] Move each operator into its own file (make an `Operator` trait?)
- [x] Add catch operator
- [] Add bytes
- [] Add stringify
- [] Add string to bytes
- [] Add bytes to string
- [] Add parse
- [] Add sockets

(func subst (from to content)
  (cond ((atom content)
         (cond ((eq from content) to)
               ('t content)))
        ('t (cons (subst from to (first content))
                  (subst from to (tail content))))))