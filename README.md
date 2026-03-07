[![Documentation on GitHub Pages](https://github.com/marcel-scherzinger/scratch-test-interpreter/actions/workflows/deploy-docs-to-pages.yml/badge.svg)](https://marcel-scherzinger.github.io/scratch-test-interpreter)
[![100% Rust](https://img.shields.io/badge/Rust-100%25-32c955?logo=rust)](https://rust-lang.org)

# About

The block-oriented programming language [Scratch](https://scratch.mit.edu/)
is used by many institutions to teach beginners how programming concepts work.
The `scratch-test` project is planned as a way to unit-test submissions of
learners for defined exercises.

This repository contains an interpreter for executing Scratch programs.
It requires an object implementing a `State` trait so the behaviour can
be extended in flexible ways

# Limitations

This project is assumed to be used for **algorithmic exercises**, so the focus is
on control structures, input, output, variables and lists.

- Sounds, movements, colors, etc. are no-ops or cancel the execution if a result
  is of them is needed
- As Scratch gives *no guarantees* about the execution order of parallel programs
  this project disallows them completly
  The usage of parallelism in a file can lead to the interpreter rejecting it.
  _(Just stick to a single green-flag event and you're fine.)_
- Scratch often tries to do _something_ to avoid exceptions or fatal errors.
  Some expressions don't evaluate to a value programmers would expect based on their
  knowledge from other languages. This project tries to model them but there is still a
  chance of differences in behaviour, especially when it comes to numbers.
