use predicates::prelude::*;
mod common;
use common::{setup_custom_project, setup_fixture};

// =============================================================================
// BASIC PREDICATE TESTS
// =============================================================================

#[test]
fn test_def_predicate_ocaml() {
    let dir = setup_fixture("ocaml_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:greet")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.ml"));
}

#[test]
fn test_func_predicate_ocaml() {
    let dir = setup_fixture("ocaml_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:add")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.ml"));
}

#[test]
fn test_str_predicate_ocaml() {
    let dir = setup_fixture("ocaml_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("str:Hello")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.ml"));
}

// =============================================================================
// COMBINATION TESTS
// =============================================================================

#[test]
fn test_ocaml_func_and_str() {
    let dir = setup_fixture("ocaml_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:greet & str:Hello")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.ml"));
}

#[test]
fn test_ocaml_or_operations() {
    let dir = setup_fixture("ocaml_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:add | func:greet")
        .assert()
        .success()
        .stdout(predicate::str::contains("main.ml"));
}

// =============================================================================
// CUSTOM PROJECT TESTS
// =============================================================================

#[test]
fn test_ocaml_custom_type() {
    let dir = setup_custom_project(&[(
        "types.ml",
        r#"
type color = Red | Green | Blue

type point = { x : float; y : float }

let distance p1 p2 =
  sqrt ((p2.x -. p1.x) ** 2.0 +. (p2.y -. p1.y) ** 2.0)

let origin = { x = 0.0; y = 0.0 }
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:distance & ext:ml")
        .assert()
        .success()
        .stdout(predicate::str::contains("types.ml"));
}

#[test]
fn test_ocaml_custom_module() {
    let dir = setup_custom_project(&[(
        "stack.ml",
        r#"
module Stack = struct
  type 'a t = 'a list

  let empty = []

  let push x s = x :: s

  let pop = function
    | [] -> None
    | x :: xs -> Some (x, xs)

  let is_empty s = s = []
end
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:push & func:pop")
        .assert()
        .success()
        .stdout(predicate::str::contains("stack.ml"));
}

#[test]
fn test_ocaml_custom_pattern_matching() {
    let dir = setup_custom_project(&[(
        "patterns.ml",
        r#"
let rec factorial n =
  match n with
  | 0 -> 1
  | n -> n * factorial (n - 1)

let rec fibonacci n =
  match n with
  | 0 -> 0
  | 1 -> 1
  | n -> fibonacci (n - 1) + fibonacci (n - 2)
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:factorial | func:fibonacci")
        .assert()
        .success()
        .stdout(predicate::str::contains("patterns.ml"));
}

#[test]
fn test_ocaml_custom_higher_order() {
    let dir = setup_custom_project(&[(
        "higher.ml",
        r#"
let rec map f = function
  | [] -> []
  | x :: xs -> f x :: map f xs

let rec filter p = function
  | [] -> []
  | x :: xs -> if p x then x :: filter p xs else filter p xs

let rec fold_left f acc = function
  | [] -> acc
  | x :: xs -> fold_left f (f acc x) xs
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:map & func:filter")
        .assert()
        .success()
        .stdout(predicate::str::contains("higher.ml"));
}

#[test]
fn test_ocaml_custom_option() {
    let dir = setup_custom_project(&[(
        "options.ml",
        r#"
let safe_divide x y =
  if y = 0.0 then None
  else Some (x /. y)

let bind opt f =
  match opt with
  | None -> None
  | Some x -> f x

let (>>=) = bind

let calculate a b c =
  safe_divide a b >>= fun x ->
  safe_divide x c
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:safe_divide & func:calculate")
        .assert()
        .success()
        .stdout(predicate::str::contains("options.ml"));
}

#[test]
fn test_ocaml_custom_functor() {
    let dir = setup_custom_project(&[(
        "functors.ml",
        r#"
module type Comparable = sig
  type t
  val compare : t -> t -> int
end

module MakeSet (Item : Comparable) = struct
  type t = Item.t list

  let empty = []

  let add x s =
    if List.exists (fun y -> Item.compare x y = 0) s
    then s
    else x :: s

  let mem x s =
    List.exists (fun y -> Item.compare x y = 0) s
end
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:add & func:mem")
        .assert()
        .success()
        .stdout(predicate::str::contains("functors.ml"));
}

// =============================================================================
// OUTPUT FORMAT TESTS
// =============================================================================

#[test]
fn test_ocaml_format_paths() {
    let dir = setup_fixture("ocaml_project");
    let output = assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--format=paths")
        .arg("func:greet")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("main.ml"));
}

#[test]
fn test_ocaml_format_markdown() {
    let dir = setup_fixture("ocaml_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--format=markdown")
        .arg("func:add")
        .assert()
        .success()
        .stdout(predicate::str::contains("```ml"));
}

// =============================================================================
// EDGE CASE TESTS
// =============================================================================

#[test]
fn test_ocaml_not_found() {
    let dir = setup_fixture("ocaml_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:nonexistent & ext:ml")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn test_ocaml_ext_filter() {
    let dir = setup_fixture("ocaml_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:. & ext:ml")
        .assert()
        .success()
        .stdout(predicate::str::contains(".rs").not());
}
