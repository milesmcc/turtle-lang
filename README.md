# turtle
A Lispy programming language that, minus a small shell, is written in itself. It's turtles all the way down. 

## TODO
- [x] Figure out why `(eq func func)` is false
- [] Figure out namespaces
- [] Move each operator into its own file (make an `Operator` trait?)
- [] Add catch operator


(func subst (from to content)
  (cond ((atom content)
         (cond ((eq from content) to)
               ('t content)))
        ('t (cons (subst from to (first content))
                  (subst from to (tail content))))))