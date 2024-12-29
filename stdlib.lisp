(def! defmacro! (macro (\ (name args body) `(def! ~name (macro (\ ~args ~body))))))

(defmacro! defun! (name args body) `(def! ~name (\ ~args ~body)))

(defmacro! while (cond body) `(if ~cond (do ~body (while ~cond ~body))))
(defun! not (x) (if x false true))
(defun! range (start end) (if (< start end) (cons start (range (+ start 1) end)) []))
(defun! map (l func) (cons (func (first l)) (map (rest l) func)))
(defun! empty? (x) (= (count x) 0))
(defun! ? (x) (if x true false))
(defun! all? (ls) (if (empty? ls) true (if (first ls) (all? (rest ls)) false)))
(defun! any? (ls) (if (empty? ls) false (if (first ls) true (any? (rest ls)))))