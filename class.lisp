## Define a new class. The first parameter is an identifier for it, and the second is a list containing alternating function names, parameters, and bodies.
## usage: (class! Dog (bark () (print "WROOF")))
## usage: (def! tucky (Dog))
## usage: (call tucky bark)
## output: WROOF
## Create a constructor, treating the first parameter as a reference to a table containing the 'self' object. Return the desired object from this function
## usage: (class! Dog (init (self name) (assoc self 'name name)))
## usage: (def! tucky (Dog "Tucky"))
## Use the self parameter in other functions to access this table.
## usage: (class! Dog (bark (self) (print (get self 'name) "says WOOF")))
## usage: (call tucky bark)
## output: Tucky says WOOF
(defmacro! class! (name body) (let* (
        func (\ (fname params body) [fname `(\ ~params ~body)])
        # apply func to each set of 3 things
        funcs (\ (data) (if (empty? data) [] (cons (func (first data) (nth data 1) (nth data 2)) (funcs (trim data 3)))))
        spread (\ (ls) (if (empty? ls) [] (cons (first (first ls)) (cons (nth (first ls) 1) (spread (rest ls))))))
        # list of fname,fbody,...,{}
        funcs (cons {} (spread (funcs body)))
        # table with the functions
        class-body (apply assoc funcs)
    ) (if (contains? class-body 'init)
        `(defun! ~name params (apply ~(get class-body 'init) (cons ~class-body params)))
        `(defun! ~name _ ~class-body)
    )
))

## Call a function on an object, providing parameters to it
## usage: (call my-object functionName)
## usage: (call my-object functionName plus any other parameters)
(defmacro! call call-args
    `((\ (o f p) (apply (get o f) (cons o p))) ~(first call-args) (quote ~(nth call-args 1)) (quote ~(trim call-args 2)))
)

(class! Dog (
    init (self name) (assoc self 'name name)
    bark (self) (print (get self 'name) "says WROOF")
    say (self msg) (print (get self 'name) "says" msg)
))

(def! tucky (Dog "Tucky"))

(call tucky bark)
(call tucky say "helo")
