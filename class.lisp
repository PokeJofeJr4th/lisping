(defmacro! class! (name body) (let* (
        func (\ (fname params body) [fname `(\ ~params ~body)])
        # apply func to each set of 3 things
        funcs (\ (data) (if (empty? data) [] (cons (func (first data) (nth data 1) (nth data 2)) (funcs (rest (rest (rest data)))))))
        spread (\ (ls) (if (empty? ls) [] (cons (first (first ls)) (cons (nth (first ls) 1) (spread (rest ls))))))
        # list of fname,fbody,...,{}
        funcs (cons {} (spread (funcs body)))
        _ (print funcs)
        # table with the functions
        class-body (apply assoc funcs)
    ) (if (contains? class-body 'init)
        `(defun! ~name params (apply ~(get class-body 'init) (cons ~class-body params)))
        `(defun! ~name _ ~class-body)
    )
))

(defmacro! call (obj func params) 
    `((\ (o f p) (apply (get o f) (cons o p))) ~obj ~['quote func] ~params)
)

(class! Dog (
    init (self name) (assoc self 'name name)
    bark (self) (print (get self 'name) "says WROOF")
))

(def! tucky (Dog "Tucky"))

(call tucky bark ())
