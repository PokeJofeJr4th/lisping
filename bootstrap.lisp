(defun! READ (x) (read_str x))

(defun! next (reader) [(first reader) (rest reader)])

(defun! read_str (s) (first (read_form
    (findall "[\\s,]*(~@|[\\[\\]{}()'`~^@]|\"(?:\\\\.|[^\\\\\"])*\"?|;.*|[^\\s\\[\\]{}('\"`,;)]*)" s))
))

# read a single atom or list
(defun! read_form (reader) (let* (
    next_char (first reader)
    _ (print "Read Form" reader)
    ) (if (= next_char "(") (read_list (rest reader)) (read_atom reader))
))

# gets a single atom from the reader
(defun! read_atom (reader) (let* (
    (atom reader) (next reader)
    atom (try* (int atom) (catch* _ (symbol atom)))
    #_ (print "Read Atom" atom)
) [atom reader]))

# takes elements until reaching a closing parenthesis
(defun! read_list (reader) (let* (
    next_char (first reader)
    #_ (print "Read List" reader)
    ) (if (= next_char ")") [[] (rest reader)] (let*
        ((form reader) (read_form reader)
        (sub_list reader) (read_list reader)
        [(cons form sub_list) reader]
    )))
))

(defun! LET* (lets ret env) (let* (
        #_ (print "let*" lets ret)
        binding (first lets)
        value (nth lets 1)
        inner-env (assoc env binding value)
        lets (rest (rest lets))
        #_ (print "let*" (count lets))
        ret (first (if (= (count lets) 0) (EVAL ret inner-env) (LET* lets ret inner-env)))
    ) [ret env]
))

(defun! EVAL (x env) (if (list? x)
    (if (= (first x) 'let*) (
        (LET* (nth x 1) (nth x 2) env)
    )
    (let* (
        _ (print "EVAL" x env)
        (func env) (EVAL (first x) env)
        #_ (print "(rest x) =" (rest x))
        #_ (print "func =" func)
        #_ (print "map ..." (map (rest x) (\ (y) (first (EVAL y env)))))
    ) (func (map (rest x) (\ (y) (first (EVAL y env)))) env))
    ) (if (symbol? x) (do (print "EVAL" x) [(if (contains? env x) (get env x) (err UnresolvedIdentifier x)) env]) [x env])
))

(defun! rep (x env) (try* (let* (
    (result env) (EVAL (READ x) env)
    _ (print result)
    ) env
) (catch* err (do (print "err" err) env))))
(defun! repl (env) (let* (
    _ (print (str "user>"))
    env (rep (input) env)
    ) (repl env)
))

(def! repl_env {
    + (\ (args env) [(apply + args) env])
    - (\ (args env) [(apply - args) env])
    * (\ (args env) [(apply * args) env])
    / (\ (args env) [(apply / args) env])
})

(repl repl_env)