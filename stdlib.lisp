(def! defmacro! (macro (\ (name args body) `(def! ~name (macro (\ ~args ~body))))))

(defmacro! defun! (name args body) `(def! ~name (\ ~args ~body)))

(defmacro! while (cond body) `(if ~cond (do ~body (while ~cond ~body))))
(defun! not (x) (if x false true))
(defun! range (start end) (if (< start end) (cons start (range (+ start 1) end)) []))
(defun! range (start end) ( let* (
        range_inner (\ (start end tmp) (if (< start end) (range_inner start (- end 1) (cons (- end 1) tmp)) tmp))
    ) (range_inner start end [])
))