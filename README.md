
<p align="center">
  <h1 align="center">Turtle Lang</h1>
  
  <br>

  <p align="center">
    A humble, fun, and friendly Lisp
    <br>
    <strong><a href="#install">Install »</a></strong>
  </p>
  <p align="center"><a href="#examples">Examples</a> &bull; <a href="TUTORIAL.md">Tutorial</a></p>
</p>

<br>

## Motivation

There are a lot of programming languages out there, and most aren't suitable for any real-world use---they are _toy programming languages_. Turtle is no exception: it exists because I wanted to experiment with Lisp without having to deal with the nuances of the most popular implementations (Common Lisp, Clojure, Scheme, etc).

Turtle has no IO facilities, and it's not particularly fast. However, it's ruthlessly simple, memory efficient, thread safe (well, at least it will be once I implement a way to spawn threads), and is well-suited for self-contained programming and math challenges (like Project Euler).

## Install

You can install Turtle using [Cargo](crates.io) by running `cargo install turtle-lang`.

## Examples

##### Hello World

```lisp
(disp "Hello, world!")
```

##### Fizz Buzz (Crackle Pop)

FizzBuzz is a simple program that counts to 100, but prints "Fizz" if a number is divisible by 3, "Buzz" if a number is divisible by 5, and "FizzBuzz" if a number is divisible by both 3 and 5.

First, let's define a function that handles a single number:

```lisp
(func handle (n)
    (cond ((eq (modulo n 15) 0) (disp "FizzBuzz"))
          ((eq (modulo n 3) 0) (disp "Fizz"))
          ((eq (modulo n 5) 0) (disp "Buzz"))
          ('t (disp n))))
```

Now, here's an imperative approach to counting: 

```lisp
(let 'i 0)
(while
    (decreasing 100 i)
    (do
        (handle i)
        (++ i)))
```

Here's a more functional approach:

```lisp
(map handle (range 100))
```

##### Prime Factorization

The [standard library](src/stdlib) has a built-in function that finds the prime factorization of any integer. Here's its imperative definition:

```lisp
(func prime-factorization 
    (n) 
    (do 
        (letq factors ()) 
        (letq curr n) 
        (while 
            (not (eq ,(append '(prod) factors) n))
            (do
                (let 'trying 2)
                (while
                    (not (eq (modulo curr trying) 0))
                    (++ trying))
                (push! factors trying)
                (let 'curr (/ curr trying))))
     factors))
```

Here's how you might go about using this definition in your own code (in this example, to solve Project Euler [problem 3](https://projecteuler.net/problem=3)):

```lisp
;; From https://projecteuler.net/problem=3
;;
;; The prime factors of 13195 are 5, 7, 13 and 29.
;;
;; What is the largest prime factor of the number 600851475143 ?

(import "@math" :math)

(let 'n 600851475143)

(assert (eq (last (math::prime-factorization n)) 6857))
```