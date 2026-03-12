let greet name =
  print_endline ("Hello " ^ name)

let add a b = a + b

let () =
  greet "world";
  print_endline (string_of_int (add 1 2))
