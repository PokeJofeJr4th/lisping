(defun! READ (x) (read_str x))
(defun! EVAL (x) x)
(defun! PRINT (x) (print x))
(defun! rep (x) (PRINT (EVAL (READ x))))

(defun! next (reader) (list (first reader) (rest reader)))

(defun! read_str (s) (first (read_form
    (findall "[\\s,]*(~@|[\\[\\]{}()'`~^@]|\"(?:\\\\.|[^\\\\\"])*\"?|;.*|[^\\s\\[\\]{}('\"`,;)]*)" s))
))
(defun! read_form (reader) (let* (
    next_char (first reader)
    #_ (print "Read Form" reader)
    ) (if (= next_char "(") (read_list (rest reader)) (read_atom reader))
))
(defun! read_atom (reader) (let* (
    atom+reader (next reader)
    #_ (print "atom+reader=" atom+reader)
    atom (nth atom+reader 0)
    reader (nth atom+reader 1)
    parse_result (int atom)
    atom (if (err? parse_result) (symbol atom) parse_result)
    #_ (print "Read Atom" atom)
) (list atom reader)))
# takes elements until reaching a closing parenthesis
(defun! read_list (reader) (let* (
    next_char (first reader)
    #_ (print "Read List" reader)
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

(while true (do (print (str "user>")) (rep (input))))