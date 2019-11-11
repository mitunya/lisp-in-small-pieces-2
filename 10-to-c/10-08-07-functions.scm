
(define-method (->C (e Closure-Creation) out)
  (format out "SCM_close")
  (between-parentheses out
    (format out "SCM_CfunctionAddress(function_~A),~A,~A"
            (Closure-Creation-index e)
            (generate-arity (Closure-Creation-variables e))
            (number-of (Closure-Creation-free e)))
    (->C (Closure-Creation-free e) out)))

(define (generate-arity variables)
  (let count ((variables variables)(arity 0))
    (if (pair? variables)
        (if (Local-Variable-dotted? (car variables))
            (- (+ arity 1))
            (count (cdr variables (+ 1 arity))))
        arity)))

(define-method (->C (e No-Free) out)
  #t)

(define-method (->C (e Free-Environment) out)
  (format out ",")
  (->C (Free-environment-first e) out)
  (->C (Free-environment-others e) out))

(define (generate-functions out definitions)
  (format out "~%/* Functions: */~%")
  (for-each (lambda (def)
              (generate-closure-structure out def)
              (generate-possibly-dotted-definition out def))
            (reverse definitions)))

(define (generate-closure-structure out definition)
  (format out "SCM_DefineClosure(function_~A, "
          (Function-Definition-index definition))
  (generate-local-temporaries (Function-Definition-free definition) out)
  (format out ");~%"))

(define (generate-possibly-dotted-definition out definition)
  (format out "~%SCM_DeclareFunction(function_~A) {~%"
          (Function-Definition-index definition))
  (let ((vars (Function-Definition-variables definition))
        (rank -1))
    (for-each (lambda (v)
                (set! rank (+ rank 1))
                (cond ((Local-Variable-dotted? v)
                       (format out "SCM_DeclareLocalDottedVariable("))
                      ((Variable? v)
                       (format out "SCM_DeclareLocalVariable(")))
                (variable->C v out)
                (format out ",~A);~%" rank))
              vars)
    (let ((temps (With-Temp-Function-Definition-temporaries definition)))
      (cond ((pair? temps)
             (generate-local-temporaries temps out)
             (format out "~%"))))
    (format out "return ")
    (->C (Function-Definition-body definition) out)
    (format out ";~%}~%~%")))

(define (generate-local-temporaries temps out)
  (cond ((pair? temps)
         (format out "SCM ")
         (variable->C (car temps) out)
         (format out "; ")
         (generate-local-temporaries (cdr temps) out))))
