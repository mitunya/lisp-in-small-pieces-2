; Alternative environment implementation
;
; I think it's fair to say that this implementation traded simplicity in
; env extension for complexity in lookup/update.
; Since I assume that an environment is more often looked up than extended,
; the original implementation seems preferable.

(import (builtin core)
        (libs utils))

(define (evaluate e env)
  (if (atom? e)
      (cond ((symbol? e) (lookup e env))
            ((or (number? e) (string? e) (char? e) (boolean? e)) e)
            (else (wrong "Cannot evaluate" e)))

      (case (car e)
        ((quote)  (cadr e))
        ((if)     (if (evaluate (cadr e) env)
                      (evaluate (caddr e) env)
                      (evaluate (cadddr e) env)))
        ((begin)  (eprogn (cdr e) env))
        ((set!)   (update! (cadr e) env (evaluate (caddr e) env)))
        ((lambda) (make-function (cadr e) (cddr e) env))
        (else     (invoke (evaluate (car e) env)
                          (evlis (cdr e) env))))))

(define (atom? exp) (not (pair? exp)))

(define wrong error)

(define (eprogn exps env)
  (if (pair? exps)
      (if (pair? (cdr exps))
          (begin (evaluate (car exps) env)
                 (eprogn (cdr exps) env))
          (evaluate (car exps) env))
      empty-begin))

(define empty-begin 'empty-begin)

(define (evlis exps env)
  (if (pair? exps)
      (let ((argument (evaluate (car exps) env)))
        (cons argument (evlis (cdr exps) env)))
      '()))

(define (extend env names values)
  (cons (cons names values) env))

(define (lookup id env)
  (define (scan names values)
    (cond ((null? names) (lookup id (cdr env)))
          ((eq? (car names) id) (car values))
          (else (scan (cdr names) (cdr values)))))
  (if (null? env)
      (wrong "No such binding" id)
      (scan (caar env) (cdar env))))

(define (update id env value)
  (define (scan names values)
    (cond ((null? names) (lookup id (cdr env)))
          ((eq? (car names) id)
           (set-car! values value)
           value)
          (else (scan (cdr names) (cdr values)))))
  (if (null? env)
      (wrong "No such binding" id)
      (scan (caar env) (cdar env))))

(define (lookup-old id env)
  (if (pair? env)
      (if (eq? (caar env) id)
          (cdar env)
          (lookup-old id (cdr env)))
      (wrong "No such binding" id)))

(define (update-old! id env value)
  (if (pair? env)
      (if (eq? (caar env) id)
          (begin (set-cdr! (car env) value)
                 value)
          (update-old! id (cdr env) value))
      (wrong "No such binding" id)))

(define (extend-old env variables values)
  (cond ((pair? variables)
         (if (pair? values)
             (cons (cons (car variables) (car values))
                   (extend-old env (cdr variables) (cdr values)))
             (wrong "Too few values")))
        ((null? variables)
         (if (null? values)
             env
             (wrong "Too many values")))
        ((symbol? variables) (cons (cons variables values) env))))

(define (invoke fn args)
  (if (procedure? fn)
      (fn args)
      (wrong "Not a function" fn)))

(define (make-function variables body env)
  (lambda (values)
    (eprogn body (extend env variables values))))

(define env.init '())

(define env.global env.init)

(define (definitial name . value)
  (if (null? value)
      (begin (set! env.global (extend env.global (list name) '(void)))
             name)
      (begin (set! env.global (extend env.global (list name) value))
             name)))

(define (defprimitive name value arity)
  (definitial name
    (lambda (values)
      (if (= arity (length values))
          (apply value values)
          (wrong "Incorrect arity" (list name values))))))

(definitial 't #t)
(definitial 'f 'the-false-value)
(definitial 'nil '())

(definitial 'foo)
(definitial 'bar)
(definitial 'fib)
(definitial 'fact)

(defprimitive 'cons cons 2)
(defprimitive 'car car 1)
(defprimitive 'cdr cdr 1)
(defprimitive 'set-car! set-car! 2)
(defprimitive 'set-cdr! set-cdr! 2)
(defprimitive '+ + 2)
(defprimitive '- - 2)
(defprimitive '* * 2)
(defprimitive '/ / 2)
(defprimitive 'eq? eq? 2)
(defprimitive '< < 2)

(define (chapter1-scheme)
  (define (toplevel)
    (display (evaluate (read) env.global))
    (toplevel))
  (toplevel))