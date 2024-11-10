(defun! READ (x) (read_str x))

(defun! next (reader) (list (first reader) (rest reader)))

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
    atom+reader (next reader)
    _ (print "atom+reader=" atom+reader)
    atom (nth atom+reader 0)
    reader (nth atom+reader 1)
    parse_result (int atom)
    atom (if (err? parse_result) (symbol atom) parse_result)
    _ (print "Read Atom" atom)
) (list atom reader)))

# takes elements until reaching a closing parenthesis
(defun! read_list (reader) (let* (
    next_char (first reader)
    _ (print "Read List" reader)
    ) (if (= next_char ")") (list (list) (rest reader)) (let*
        (form+reader (read_form reader)
        form (nth form+reader 0)
        reader (nth form+reader 1)
        sub_list+reader (read_list reader)
        sub_list (nth sub_list+reader 0)
        reader (nth sub_list+reader 1))
        (list (cons form sub_list) reader)
    ))
))

(defun! LET* (lets ret env) (let* (
        _ (print "let*" lets ret)
        binding (first lets)
        value (nth lets 1)
        inner-env (assoc env binding value)
        lets (rest (rest lets))
        #_ (print "let*" (count lets))
        ret (first (if (= (count lets) 0) (EVAL ret inner-env) (LET* lets ret inner-env)))
    ) (list ret env)
))

(defun! EVAL (x env) (if (list? x)
    (if (= (first x) 'let*) (
        (LET* (nth x 1) (nth x 2) env)
    )
    (let* (
        _ (print "EVAL" x env)
        func+env (EVAL (first x) env)
        _ (print "func+env=" func+env)
        env (nth func+env 1)
        func (first func+env)
        _ (print "(rest x)" (rest x))
        _ (print "map ..." (map (rest x) (\ (y) (first (EVAL y env)))))
    ) (func (map (rest x) (\ (y) (first (EVAL y env)))) env))
    ) (if (symbol? x) (list (get env x) env) (list x env))
))

(defun! rep (x env) (let* (
    result+env (EVAL (READ x) env)
    _ (print (first result+env))
    ) (nth result+env 1)
))
(defun! repl (env) (let* (
    _ (print (str "user>"))
    env (rep (input) env)
    ) (repl env)
))

(def! repl_env {
    + (\ (args env) (list (apply + args) env))
    - (\ (args env) (list (apply - args) env))
    * (\ (args env) (list (apply * args) env))
    / (\ (args env) (list (apply / args) env))
    err (\ (args env) (list (cons 'err args) env))
})

(repl repl_env)