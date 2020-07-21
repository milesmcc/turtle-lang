;;; -*- Mode: Lisp; Syntax: Turtle -*-

(label 'setq (macro (identifier value) (label identifier ,value))))