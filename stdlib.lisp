# define a macro to perform a transformation on syntax
# usage: (defmacro! my-macro! (my macro parameters) (construct the resulting syntax))
(def! defmacro! (macro (\ (name args body) `(def! ~name (macro (\ ~args ~body))))))

(defmacro! defun! (name args body) `(def! ~name (\ ~args ~body)))

(defmacro! while (cond body) `(if ~cond (do ~body (while ~cond ~body))))
(defun! not (x) (if x false true))
(defun! range (start end) (if (< start end) (cons start (range (+ start 1) end)) []))
(defun! map (l func) (cons (func (first l)) (map (rest l) func)))
(defun! empty? (x) (= (count x) 0))
# convert a value to a boolean
(defun! ? (x) (if x true false))
# returns true if the list is empty or if every value of the list is truthy
(defun! all? (ls) (if (empty? ls) true (if (first ls) (all? (rest ls)) false)))
# return true if any value in the list is truthy
(defun! any? (ls) (if (empty? ls) false (if (first ls) true (any? (rest ls)))))
# return a copy of the provided list with a given number of values removed from the start
(defun! trim (ls i) (if (= i 0) ls (trim (rest ls) (- i 1))))