## Define a macro to perform a transformation on syntax
## usage: (defmacro! my-macro! (my macro parameters) (construct the resulting syntax))
(def! defmacro! (macro (\ (name args body) `(def! ~name (macro (\ ~args ~body))))))

## Create a function.
## usage: (defun! my-function (my function parameters) (evaluate the return value))
(defmacro! defun! (name args body) `(def! ~name (\ ~args ~body)))

## Evaluate and discard the result of an expression until a condition is met
(defmacro! while (cond body) `(if ~cond (do ~body (while ~cond ~body))))

## Return false if the provided value is truthy, and true otherwise
(defun! not (x) (if x false true))

## Create a range of numbers from the start to the end
(defun! range (start end) (if (< start end) (cons start (range (+ start 1) end)) []))

## Apply a function to each element of a list
(defun! map (l func) (cons (func (first l)) (map (rest l) func)))

## Check if a table, list, or string is empty
(defun! empty? (x) (= (count x) 0))

## Convert a value to a boolean
(defun! ? (x) (if x true false))

## Return true if the list is empty or if every value of the list is truthy
(defun! all? (ls) (if (empty? ls) true (if (first ls) (all? (rest ls)) false)))

## Return true if any value in the list is truthy
(defun! any? (ls) (if (empty? ls) false (if (first ls) true (any? (rest ls)))))

## Return a copy of the provided list with a given number of values removed from the start
(defun! trim (ls i) (if (= i 0) ls (trim (rest ls) (- i 1))))