;;; Bytecode object file for 08-reflection/reflective-interpreter.scm
;;; Dynamic variables
()

;;; Global modifiable variables
()

;;; Quotations
#(it extend error global-env toplevel eval evlis eprogn reference (set! extend (lambda (env names values) (if (pair? names) (if (pair? values) ((lambda (newenv) (begin (set-variable-value! (car names) newenv (car values)) (extend newenv (cdr names) (cdr values)))) (enrich env (car names))) (error "Too few arguments" names)) (if (symbol? names) ((lambda (newenv) (begin (set-variable-value! names newenv values) newenv)) (enrich env names)) (if (null? names) (if (null? values) env (error "Too much arguments" values)) env))))) (set! eval (lambda (e r) (if (pair? e) ((lambda (f) (if (flambda? f) (flambda-apply f r (cdr e)) (apply f (evlis (cdr e) r)))) (eval (car e) r)) (if (symbol? e) (reference e r) e)))) (set! evlis (lambda (e* r) (if (pair? e*) ((lambda (v) (cons v (evlis (cdr e*) r))) (eval (car e*) r)) (quote ())))) (set! eprogn (lambda (e+ r) (if (pair? (cdr e+)) (begin (eval (car e+) r) (eprogn (cdr e+) r)) (eval (car e+) r)))) (set! reference (lambda (name r) (if (variable-defined? name r) (variable-value name r) (if (variable-defined? name global-env) (variable-value name global-env) (error "No such variable" name))))) (eval form r) "No such variable" ((quote (local 0 . 0)) (if (local 0 . 1)) (set! (local 0 . 2)) (lambda (local 0 . 3)) (flambda (local 0 . 4)) (monitor (local 0 . 5)) (it (local 1 . 0)) (extend (local 1 . 1)) (error (local 1 . 2)) (global-env (local 1 . 3)) (toplevel (local 1 . 4)) (eval (local 1 . 5)) (evlis (local 1 . 6)) (eprogn (local 1 . 7)) (reference (local 1 . 8)) (exit (local 2 . 0)) (prompt-in (local 3 . 0)) (prompt-out (local 3 . 1)) (make-toplevel (local 4 . 0)) (make-flambda (local 4 . 1)) (flambda? (local 4 . 2)) (flambda-apply (local 4 . 3))) "?? " "== " make-toplevel 98127634)

;;; Bytecode
#(245 20 247 38 43 19 27 34 40 2 28 162 2 75 5 32 40 2 28 139 2 73 32 19 26 34 40 2 28 123 2 72 32 40 2 30 13 73 32 6 1 0 34 2 34 51 60 39 46 43 246 9 0 34 9 1 34 9 2 34 9 3 34 9 4 34 9 5 34 9 6 34 9 7 34 9 8 34 55 10 64 8 64 7 64 6 64 5 64 4 63 62 61 60 32 9 9 40 2 30 26 73 32 6 2 0 34 19 23 34 1 34 2 34 52 61 60 39 37 45 38 34 51 60 39 46 43 23 40 2 30 96 72 32 1 26 1 3 6 3 0 94 19 24 34 50 39 37 45 38 34 51 60 32 19 25 34 1 34 51 60 39 37 45 38 31 14 6 3 0 34 1 34 51 60 39 37 45 38 30 17 6 2 5 34 1 34 6 2 3 34 52 61 60 39 37 45 38 33 34 51 60 32 1 26 2 0 6 4 1 94 1 94 89 33 6 1 4 34 6 1 3 34 51 60 39 46 43 25 4 9 10 9 11 9 12 9 13 6 3 1 34 40 2 30 4 73 32 2 43 34 51 60 39 37 45 38 34 6 3 1 34 40 2 30 37 75 5 32 6 1 5 34 6 1 5 34 2 34 1 34 52 61 60 39 37 45 38 31 3 3 30 1 4 34 1 34 52 61 60 39 46 43 34 51 60 39 37 45 38 34 6 3 1 34 40 2 30 107 74 32 9 14 34 51 60 32 19 31 34 6 1 1 34 6 1 0 34 52 61 60 39 37 45 38 31 21 19 30 34 6 1 1 34 6 1 0 34 1 34 53 62 61 60 39 46 30 57 19 31 34 6 1 1 34 6 2 3 34 52 61 60 39 37 45 38 31 21 19 30 34 6 1 1 34 6 2 3 34 1 34 53 62 61 60 39 46 30 16 6 2 2 34 9 15 34 6 1 1 34 52 61 60 39 46 43 34 51 60 39 37 45 38 34 6 3 1 34 40 2 30 52 78 3 44 2 32 40 2 30 42 78 1 44 0 32 6 2 7 34 6 1 2 34 6 2 1 34 6 1 0 34 6 1 1 34 1 34 53 62 61 60 39 37 45 38 34 52 61 60 39 46 43 43 34 51 60 39 37 45 38 34 6 3 1 34 40 2 30 65 78 3 44 2 32 6 4 1 34 40 2 30 46 78 2 44 1 32 6 2 7 34 6 1 2 34 6 2 1 34 6 1 0 34 6 1 1 34 1 34 2 35 100 34 53 62 61 60 39 37 45 38 34 52 61 60 39 46 43 34 51 60 39 46 43 34 51 60 39 37 45 38 34 6 3 1 34 40 2 30 38 78 3 44 2 32 6 1 5 34 2 34 1 34 52 61 60 39 37 45 38 246 6 1 7 34 3 34 1 34 52 61 60 39 37 45 38 247 43 34 51 60 39 37 45 38 34 55 7 64 5 64 4 63 62 61 60 32 6 1 4 34 9 16 254 34 51 60 39 37 45 38 33 33 247 43 34 51 60 39 46 43 21 1 34 9 17 34 9 18 34 52 61 60 39 46 43 34 9 19 34 9 20 34 56 1 47 0 32 19 23 34 40 2 30 10 72 32 6 1 0 34 1 35 100 43 34 40 2 30 18 72 32 1 92 31 10 1 90 34 6 1 0 35 104 30 1 11 43 34 40 2 30 19 74 32 19 27 34 1 91 34 2 34 3 34 53 62 61 60 39 46 43 34 53 62 61 60 39 37 45 38 33 34 53 62 61 60 39 46 43)

;;; Entry point
5