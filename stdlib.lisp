## Define a macro to perform a transformation on syntax
## usage: (defmacro! my-macro! (my macro parameters) (construct the resulting syntax))
(def! defmacro! (macro (\ (name args body) `(def! ~name (macro (\ ~args ~body))))))

## Create a function.
## usage: (defun! my-function (my function parameters) (evaluate the return value))
(defmacro! defun! (name args body) `(def! ~name (\ ~args ~body)))

## Concatenate all of the arguments into a list
(defun! list x x)

## Evaluate and discard the result of an expression until a condition is met
(defmacro! while (cond body) `(if ~cond (do ~body (while ~cond ~body))))

## Return false if the provided value is truthy, and true otherwise
(defun! not (x) (if x false true))

## Convert the provided value to a boolean
(defun! bool (x) (if x true false))

## Create a range of numbers from the start to the end
(defun! range (start end) (if (< start end) (cons start (range (+ start 1) end)) []))

## Create a list with the given element repeated the given number of times
(defun! repeat (x l) (if (<= l 0) [] (cons x (repeat x (- l 1)))))

## Apply a function to each element of a list
(defun! map (l func) (cons (func (first l)) (map (rest l) func)))

## Check if a table, list, or string is empty
(defun! empty? (x) (= (count x) 0))

## Return true if the list is empty or if every of the parameters is truthy
(defun! all? ls (if (empty? ls) true (if (first ls) (all? (rest ls)) false)))

## Return true if any of the parameters is truthy
(defun! any? ls (if (empty? ls) false (if (first ls) true (any? (rest ls)))))

## Return a copy of the provided list with a given number of values removed from the start
(defun! trim (ls i) (if (any? (= i 0) (empty? ls)) ls (trim (rest ls) (- i 1))))

## Calculate a factorial
(defun! fact (n) (apply * (cons 1 (range 1 (+ 1 n)))))

## Calculate the sum of all elements in a list
(defun! sum (n) (apply + n))

## Calculate the product of all elements in a list
(defun! prod (n) (apply * n))

# ## Calculate the first number raised to the power of the second number
# (defun! pow (x y) (apply * (cons 1 (repeat x y))))

## Calculate the first number raised to the power of the second number
(defun! pow (x y) (let* (
    pow_helper (\ (a x y) (if (= y 0) a (pow_helper (if (odd? y) (* a x) a) (* x x) (/ y 2))))
) (pow_helper 1 x y)))

## Check if a number is even
(defun! even? (x) (= (% x 2) 0))

## Check if a number is odd
(defun! odd? (x) (= (% x 2) 1))

## Take the combinations between two numbers
(defun! choose (a b) (
    /
    (apply * (range (+ b 1) (+ a 1)))
    (fact (- a b))
))
