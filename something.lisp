list
(+ 2 (* 3 4))
(str 'hello (chr 32) 'world)
((\ (x) (+ x 2)) 10)
(list 1 2 (+ 4 (\ (x) (+ x 2))) '(1 2 3 4))
`(1 2 3 4 ~(+ 2 3) 6)
((\ (x) (if (int? x) (+ x 1) (err NotANumber x))) 4)