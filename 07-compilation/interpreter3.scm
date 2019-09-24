(import (builtin core)
        (libs utils)
        (libs book)
        (06-fast-interpreter common))

; instructions are encoded as bytecode

(include "pretreatment.scm")

(define backup.g.current g.current)
(define (original.g.current) backup.g.current)

(define *code* 'anything)
(define *pc* 'anything)
(define *constants* 'anything)
(define *env* 'anything)
(define *quotations* 'anything)
(define *stack* 'anything)
(define *stack-index* 'anything)
(define *val* 'anything)
(define *fun* 'anything)
(define *arg1* 'anything)
(define *arg2* 'anything)
(define *exit* 'anything)
(define finish-pc 'anything)


; ===========================================================================
(define (invoke f tail?)
  (cond ((closure? f) (if (not tail?)
                          (stack-push *pc*))
                      (set! *env* (closure-closed-environment f))
                      (set! *pc* (closure-code f)))
        ((primitive? f) (if (not tail?)
                            (stack-push *pc*))
                        ((primitive-address f)))
        (else (signal-exception #f (list "Not a function" f)))))

; ===========================================================================

;(define (CALL0 address)
;  (append m1 (INVOKE1 address)))

(define (CALL1 address m1)
  (println 'CALL1 address m1)
  (append m1 (INVOKE1 address)))

(define (CALL2 address m1 m2)
  (append m1 (PUSH-VALUE) m2 (POP-ARG1) (INVOKE2 address)))

(define (CALL3 address m1 m2 m3)
  (append m1 (PUSH-VALUE)
          m2 (PUSH-VALUE)
          m3 (POP-ARG2) (POP-ARG1)
          (INVOKE3 address)))

; ===========================================================================
(define (run)
  (println "OP:" (instruction-decode *code* *pc*))
  (let ((instruction (fetch-byte)))
    ((vector-ref instruction-body-table instruction)))
  (if (not *exit*)  ; workaround for interpreter without call/cc
      (run)
      *val*))

(define (instruction-size code pc)
  (let ((instruction (vector-ref code pc)))
    (vector-ref instruction-size-table instruction)))

(define (instruction-decode code pc)
  (define (decode-fetch-byte)
    (let ((byte (vector-ref code pc)))
      (set! pc (+ pc 1))
      byte))
  (let ((instruction (decode-fetch-byte)))
    ((vector-ref instruction-decode-table instruction) decode-fetch-byte)))

(define (fetch-byte)
  (let ((byte (vector-ref *code* *pc*)))
    (set! *pc* (+ *pc* 1))
    byte))

(define instruction-body-table (make-vector 256))
(define instruction-size-table (make-vector 256))
(define instruction-decode-table (make-vector 256))

(define-syntax run-clause
  (syntax-rules ()
    ((run-clause () body) (begin . body))
    ((run-clause (a) body) (let ((a (fetch-byte)))
                             . body))
    ((run-clause (a b) body) (let ((a (fetch-byte))
                                   (b (fetch-byte)))
                               . body))))


(define-syntax size-clause
  (syntax-rules ()
    ((size-clause ())    1)
    ((size-clause (a))   2)
    ((size-clause (a b)) 3)))

(define-syntax decode-clause
  (syntax-rules ()
    ((decode-clause fetcher iname ()) '(iname))
    ((decode-clause fetcher iname (a)) (let ((a (fetcher)))
                                          (list 'iname a)))
    ((decode-clause fetcher iname (a b)) (let ((a (fetcher))
                                               (b (fetcher)))
                                             (list 'iname a b)))))

(define-syntax define-instruction
  (syntax-rules ()
    ((define-instruction (name . args) n . body)
     (begin
       (vector-set! instruction-body-table n (lambda () (run-clause args body)))
       (vector-set! instruction-size-table n (size-clause args))
       (vector-set! instruction-decode-table n (lambda (fetcher) (decode-clause fetcher name args)))))))

(define (check-byte j)
  (if (or (< j 0) (> j 255))
      (static-wrong "Cannot pack this number within a byte" j)))

(define (SHALLOW-ARGUMENT-REF j)
  (check-byte j)
  (case j
    ((0 1 2 3) (list (+ 1 j)))
    (else      (list 5 j))))

(define-instruction (SHALLOW-ARGUMENT-REF0) 1
  (set! *val* (activation-frame-argument *env* 0)))
(define-instruction (SHALLOW-ARGUMENT-REF1) 2
  (set! *val* (activation-frame-argument *env* 1)))
(define-instruction (SHALLOW-ARGUMENT-REF2) 3
  (set! *val* (activation-frame-argument *env* 2)))
(define-instruction (SHALLOW-ARGUMENT-REF3) 4
  (set! *val* (activation-frame-argument *env* 3)))
(define-instruction (SHALLOW-ARGUMENT-REF j) 5
  (set! *val* (activation-frame-argument *env* j)))

(define (SET-SHALLOW-ARGUMENT! j)
  (case j
    ((0 1 2 3) (list (+ 21 j)))
    (else      (list 25 j))))

(define-instruction (SET-SHALLOW-ARGUMENT!2) 21
  (set-activation-frame-argument! *env* 0 *val*))

(define-instruction (SET-SHALLOW-ARGUMENT!1) 22
  (set-activation-frame-argument! *env* 1 *val*))

(define-instruction (SET-SHALLOW-ARGUMENT!2) 23
  (set-activation-frame-argument! *env* 2 *val*))

(define-instruction (SET-SHALLOW-ARGUMENT!3) 24
  (set-activation-frame-argument! *env* 3 *val*))

(define-instruction (SET-SHALLOW-ARGUMENT! j) 25
  (set-activation-frame-argument! *env* j *val*))

(define (DEEP-ARGUMENT-REF i j) (list 6 i j))
(define (SET-DEEP-ARGUMENT! i j) (list 26 i j))

(define-instruction (DEEP-ARGUMENT-REF i j) 6
  (set! *val* (deep-fetch *env* i j)))

(define-instruction (SET-DEEP-ARGUMENT! i j) 26
  (deep-update! *env* i j *val*))

(define (GLOBAL-REF i) (list 7 i))
(define (CHECKED-GLOBAL-REF i) (list 8 i))
(define (SET-GLOBAL! i) (list 27 i))

(define-instruction (GLOBAL-REF i) 7
  (set! *val* (global-fetch i)))

(define-instruction (CHECKED-GLOBAL-REF i) 8
  (set! *val* (global-fetch i))
  (if (eq? *val* undefined-value)
      (signal-exception #t (list "Uninitialized global variable" i))))

(define-instruction (SET-GLOBAL! i) 27
  (global-update! i *val*))

(define (PREDEFINED i)
  (check-byte i)
  (case i
    ((0 1 2 3 4 5 6 7 8) (list (+ 10 i)))
    (else                (list 19 i))))

(define-instruction (PREDEFINED0) 10 (set! *val* #t))
(define-instruction (PREDEFINED1) 11 (set! *val* #f))
(define-instruction (PREDEFINED2) 12 (set! *val* '()))
(define-instruction (PREDEFINED3) 13 (set! *val* cons))
(define-instruction (PREDEFINED4) 14 (set! *val* car))
(define-instruction (PREDEFINED5) 15 (set! *val* cdr))
(define-instruction (PREDEFINED6) 16 (set! *val* pair?))
(define-instruction (PREDEFINED7) 17 (set! *val* symbol?))
(define-instruction (PREDEFINED8) 18 (set! *val* eq?))
(define-instruction (PREDEFINED i) 19
  (set! *val* (predefined-fetch i)))

(define (CONSTANT value)
  (cond ((eq? value #t)     (list 10))
        ((eq? value #f)     (list 11))
        ((eq? value '())    (list 12))
        ((equal? value -1)  (list 80))
        ((equal? value 0)   (list 81))
        ((equal? value 1)   (list 82))
        ((equal? value 2)   (list 83))
        ((equal? value 4)   (list 84))
        ((and (integer? value)
              (<= 0 value)
              (< value 255))
         (list 79 value))
        (else (EXPLICIT-CONSTANT value))))

(define (EXPLICIT-CONSTANT value)
  (set! *quotations* (append *quotations* (list value)))
  (list 9 (- (length *quotations*) 1)))
(define-instruction (CONSTANT i) 9 (set! *val* (quotation-fetch i)))

(define-instruction (CONSTANT-1) 80 (set! *val* -1))
(define-instruction (CONSTANT0) 81 (set! *val* 0))
(define-instruction (CONSTANT1) 82 (set! *val* 1))
(define-instruction (CONSTANT2) 83 (set! *val* 2))
(define-instruction (CONSTANT4) 84 (set! *val* 4))
(define-instruction (SHORT-NUMBER value) 79 (set! *val* value))

(define (GOTO offset)
  (cond ((< offset 255) (list 30 offset))
        ((< offset (+ 255 (* 255 256)))
         (let ((offset1 (modulo offset 256)))
              ((offset2 (quotient offset 256)))
           (list 28 offset1 offset2)))
        (else (static-wrong "too long jump" offset))))

(define (JUMP-FALSE offset)
  (cond ((< offset 255) (list 31 offset))
        ((< offset (+ 255 (* 255 256)))
         (let ((offset1 (modulo offset 256)))
              ((offset2 (quotient offset 256)))
           (list 29 offset1 offset2)))
        (else (static-wrong "too long jump" offset))))

(define-instruction (SHORT-GOTO offset) 30
  (set! *pc* (+ *pc* offset)))

(define-instruction (SHORT-JUMP-FALSE offset) 31
  (if (not *val*) (set! *pc* (+ *pc* offset))))

(define-instruction (LONG-GOTO offset1 offset2) 28
  (let ((offset (+ offset1 (* 256 offset2))))
    (set! *pc* (+ *pc* offset))))

(define-instruction (LONG-JUMP-FALSE offset1 offset2) 29
  (let ((offset (+ offset1 (* 256 offset2))))
    (if (not *val*) (set! *pc* (+ *pc* offset)))))

(define (ALLOCATE-FRAME size)
  (case size
    ((0 1 2 3 4) (list (+ 50 size)))
    (else        (list 55 (+ size 1)))))

(define-instruction (ALLOCATE-FRAME1) 50
  (set! *val* (allocate-activation-frame 1)))

(define-instruction (ALLOCATE-FRAME2) 51
  (set! *val* (allocate-activation-frame 2)))

(define-instruction (ALLOCATE-FRAME3) 52
  (set! *val* (allocate-activation-frame 3)))

(define-instruction (ALLOCATE-FRAME4) 53
  (set! *val* (allocate-activation-frame 4)))

(define-instruction (ALLOCATE-FRAME5) 54
  (set! *val* (allocate-activation-frame 5)))

(define-instruction (ALLOCATE-FRAME size+1) 55
  (set! *val* (allocate-activation-frame size+1)))

(define (POP-FRAME! rank)
  (case rank
    ((0 1 2 3)  (list (+ 60 rank)))
    (else       (list 64 rank))))

(define-instruction (POP-FRAME!0) 60
  (set-activation-frame-argument! *val* 0 (stack-pop)))

(define-instruction (POP-FRAME!1) 61
  (set-activation-frame-argument! *val* 1 (stack-pop)))

(define-instruction (POP-FRAME!2) 62
  (set-activation-frame-argument! *val* 2 (stack-pop)))

(define-instruction (POP-FRAME!3) 63
  (set-activation-frame-argument! *val* 3 (stack-pop)))

(define-instruction (POP-FRAME! rank) 64
  (set-activation-frame-argument! *val* rank (stack-pop)))

(define (INVOKE1 address)
  (case address
    ((car)      (list 90))
    ((cdr)      (list 91))
    ((pair?)    (list 92))
    ((symbol?)  (list 93))
    ((display)  (list 94))
    (else (static-wrong "Cannot integrate" address))))

(define-instruction (CALL1-car) 90
  (set! *val* (car *val*)))

(define-instruction (CALL1-cdr) 91
  (set! *val* (cdr *val*)))

(define-instruction (CALL1-pair?) 92
  (set! *val* (pair? *val*)))

(define-instruction (CALL1-symbol?) 93
  (set! *val* (symbol? *val*)))

(define-instruction (CALL1-display) 94
  (set! *val* (display *val*)))

(define (ARITY=? arity+1)
  (case arity+1
    ((1 2 3 4) (list (+ 70 arity+1)))
    (else      (list 75 arity+1))))

(define-instruction (ARITY=?1) 71
  (if (not (= (activation-frame-argument-length *val*) 1))
      (signal-exception #f (list "Incorrect arity for nullary function"))))

(define-instruction (ARITY=?2) 72
  (if (not (= (activation-frame-argument-length *val*) 2))
      (signal-exception #f (list "Incorrect arity for unary function"))))

(define-instruction (ARITY=?3) 73
  (if (not (= (activation-frame-argument-length *val*) 3))
      (signal-exception #f (list "Incorrect arity for binary function"))))

(define-instruction (ARITY=?4) 74
  (if (not (= (activation-frame-argument-length *val*) 4))
      (signal-exception #f (list "Incorrect arity for ternary function"))))

(define-instruction (ARITY=? arity+1) 75
  (if (not (= (activation-frame-argument-length *val*) arity+1))
      (signal-exception #f (list "Incorrect arity"))))

(define (EXTEND-ENV) (list 32))
(define-instruction (EXTEND-ENV) 32
  (set! *env* (sr-extend* *env* *val*)))

(define (UNLINK-ENV) (list 33))
(define-instruction (UNLINK-ENV) 33
  (set! *env* (activation-frame-next *env*)))

(define (PUSH-VALUE) (list 34))
(define-instruction (PUSH-VALUE) 34
  (stack-push *val*))

(define (POP-ARG1) (list 35))
(define-instruction (POP-ARG1) 35
  (set! *arg1* (stack-pop)))

(define (POP-ARG2) (list 36))
(define-instruction (POP-ARG2) 36
  (set! *arg2* (stack-pop)))

(define (PRESERVE-ENV) (list 37))
(define-instruction (PRESERVE-ENV) 37
  (preserve-environment))

(define (RESTORE-ENV) (list 38))
(define-instruction (RESTORE-ENV) 38
  (restore-environment))

(define (POP-FUNCTION) (list 39))
(define-instruction (POP-FUNCTION) 39
  (set! *fun* (stack-pop)))

(define (CREATE-CLOSURE offset) (list 40 offset))
(define-instruction (CREATE-CLOSURE offset) 40
  (set! *val* (make-closure (+ *pc* offset) *env*)))

(define (RETURN) (list 43))
(define-instruction (RETURN) 43
  (set! *pc* (stack-pop)))

(define (FUNCTION-INVOKE) (list 45))
(define-instruction (FUNCTION-INVOKE) 45
  (invoke *fun* #f))

(define (FUNCTION-GOTO) (list 46))
(define-instruction (FUNCTION-GOTO) 46
  (invoke *fun* #t))

(define (FINISH) (list 20))
(define-instruction (FINISH) 20
  ;(*exit* *val*))
  (set! *exit* #t))

; ===========================================================================

(define (quotation-fetch i)
  (vector-ref *constants* i))

(define (preserve-environment)
  (stack-push *env*))

(define (restore-environment)
  (set! *env* (stack-pop)))

; ===========================================================================

(define (stack-push v)
  (vector-set! *stack* *stack-index* v)
  (set! *stack-index* (+ *stack-index* 1)))

(define (stack-pop)
  (set! *stack-index* (- *stack-index* 1))
  (vector-ref *stack* *stack-index*))

(define (make-closure code closed-environment)
  (list 'closure code closed-environment))

(define (closure? obj)
  (eq? (car obj) 'closure))

(define (closure-code obj)
  (cadr obj))

(define (closure-closed-environment obj)
  (caddr obj))

; ===========================================================================


(define (defprimitive name value arity)
  (case arity
    ((0) (defprimitive0 name value))
    ((1) (defprimitive1 name value))
    ((2) (defprimitive2 name value))
    ((3) (defprimitive3 name value))
    (else static-wrong "Unsupported primitive arity" name arity)))

(define (defprimitive0 name value)
  (definitial name
    (let* ((arity+1 (+ 0 1))
           (behavior
             (lambda (v* sr)
               (if (= arity+1 (activation-frame-argument-length v*))
                   (value)
                   (wrong "Incorrect arity" name)))))
      (description-extend! name `(function ,value))
      (make-closure behavior sr.init))))

(define (defprimitive1 name value)
  (definitial name
    (let* ((arity+1 (+ 1 1))
           (behavior
             (lambda (v* sr)
               (if (= arity+1 (activation-frame-argument-length v*))
                   (value (activation-frame-argument v* 0))
                   (wrong "Incorrect arity" name)))))
      (description-extend! name `(function ,value a))
      (make-closure behavior sr.init))))

(define (defprimitive2 name value)
  (definitial name
    (let* ((arity+1 (+ 2 1))
           (behavior
             (lambda (v* sr)
               (if (= arity+1 (activation-frame-argument-length v*))
                   (value (activation-frame-argument v* 0)
                          (activation-frame-argument v* 1))
                   (wrong "Incorrect arity" name)))))
      (description-extend! name `(function ,value a b))
      (make-closure behavior sr.init))))

(define (defprimitive3 name value)
  (definitial name
    (let* ((arity+1 (+ 3 1))
           (behavior
             (lambda (v* sr)
               (if (= arity+1 (activation-frame-argument-length v*))
                   (value (activation-frame-argument v* 0)
                          (activation-frame-argument v* 1)
                          (activation-frame-argument v* 2))
                   (wrong "Incorrect arity" name)))))
      (description-extend! name `(function ,value a b c))
      (make-closure behavior sr.init))))

(definitial 't #t)
(definitial 'f #f)
(definitial 'nil '())
(defprimitive 'cons 'cons 2)
(defprimitive 'car 'car 1)
(defprimitive 'cdr 'cdr 1)
(defprimitive 'pair? 'pair? 1)
(defprimitive 'symbol? 'symbol? 1)
(defprimitive 'eq? 'eq? 2)
(defprimitive 'null? 'null? 1)
(defprimitive '= = 2)
(defprimitive '< < 2)
(defprimitive '<= <= 2)
(defprimitive '> > 2)
(defprimitive '>= >= 2)
(defprimitive '+ + 2)
(defprimitive '- - 2)
(defprimitive '* * 2)
(defprimitive '/ / 2)

; ===========================================================================

(define (chapter7d-interpreter)
  (define (toplevel)
    (display ((stand-alone-producer7d (read)) 100))
    (toplevel))
  (toplevel))

(define (stand-alone-producer7d e)
  (set! g.current (original.g.current))
  (set! *quotations* '())
  (let* ((code (make-code-segment (meaning e r.init #t)))
         (start-pc (length (code-prologue)))
         (global-names (map car (reverse g.current)))
         (constants (apply vector *quotations*)))
    (lambda (stack-size)
      (run-machine stack-size start-pc code
                   constants global-names))))

(define (make-code-segment m)
  (apply vector (append (code-prologue) m (RETURN))))

(define (code-prologue)
  (set! finish-pc 0)
  (FINISH))

(define (run-machine stack-size pc code constants global-names)
  (set! sg.current (make-vector (length global-names) undefined-value))
  (set! sg.current.names global-names)
  (set! *constants* constants)
  (set! *code* code)
  (set! *env* sr.init)
  (set! *stack* (make-vector stack-size))
  (set! *stack-index* 0)
  (set! *val* 'anything)
  (set! *fun* 'anything)
  (set! *arg1* 'anything)
  (set! *arg2* 'anything)
  (stack-push finish-pc)
  (set! *pc* pc)
  (set! *exit* #f)
  ;(call/cc (lambda (exit)
  ;           (set! *exit* exit)
  ;           (run)))
  (run))

;(chapter7d-interpreter)
