use predicates::prelude::*;
mod common;
use common::{setup_custom_project, setup_fixture};

// =============================================================================
// BASIC PREDICATE TESTS
// =============================================================================

#[test]
fn test_def_predicate_haskell() {
    let dir = setup_fixture("haskell_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:greet")
        .assert()
        .success()
        .stdout(predicate::str::contains("Main.hs"));
}

#[test]
fn test_func_predicate_haskell() {
    let dir = setup_fixture("haskell_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:add")
        .assert()
        .success()
        .stdout(predicate::str::contains("Main.hs"));
}

#[test]
fn test_call_predicate_haskell() {
    let dir = setup_fixture("haskell_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("call:greet")
        .assert()
        .success()
        .stdout(predicate::str::contains("Main.hs"));
}

#[test]
fn test_str_predicate_haskell() {
    let dir = setup_fixture("haskell_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("str:Hello")
        .assert()
        .success()
        .stdout(predicate::str::contains("Main.hs"));
}

// =============================================================================
// COMBINATION TESTS
// =============================================================================

#[test]
fn test_haskell_func_and_call() {
    let dir = setup_fixture("haskell_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:greet & call:greet")
        .assert()
        .success()
        .stdout(predicate::str::contains("Main.hs"));
}

#[test]
fn test_haskell_or_operations() {
    let dir = setup_fixture("haskell_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:add | func:greet")
        .assert()
        .success()
        .stdout(predicate::str::contains("Main.hs"));
}

// =============================================================================
// CUSTOM PROJECT TESTS
// =============================================================================

#[test]
fn test_haskell_custom_type() {
    let dir = setup_custom_project(&[(
        "Types.hs",
        r#"
module Types where

data Color = Red | Green | Blue deriving (Show, Eq)

data Point = Point { x :: Double, y :: Double } deriving (Show)

distance :: Point -> Point -> Double
distance p1 p2 = sqrt ((x p2 - x p1)^2 + (y p2 - y p1)^2)
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:distance & ext:hs")
        .assert()
        .success()
        .stdout(predicate::str::contains("Types.hs"));
}

#[test]
fn test_haskell_custom_typeclass() {
    let dir = setup_custom_project(&[(
        "Classes.hs",
        r#"
module Classes where

class Printable a where
    toString :: a -> String

data Person = Person String Int

instance Printable Person where
    toString (Person name age) = name ++ " (" ++ show age ++ ")"
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:toString & ext:hs")
        .assert()
        .success()
        .stdout(predicate::str::contains("Classes.hs"));
}

#[test]
fn test_haskell_custom_monad() {
    let dir = setup_custom_project(&[(
        "Monads.hs",
        r#"
module Monads where

import Control.Monad

safeDivide :: Double -> Double -> Maybe Double
safeDivide _ 0 = Nothing
safeDivide x y = Just (x / y)

calculate :: Double -> Double -> Double -> Maybe Double
calculate a b c = do
    x <- safeDivide a b
    y <- safeDivide x c
    return y
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:safeDivide & func:calculate")
        .assert()
        .success()
        .stdout(predicate::str::contains("Monads.hs"));
}

#[test]
fn test_haskell_custom_recursion() {
    let dir = setup_custom_project(&[(
        "Recursion.hs",
        r#"
module Recursion where

factorial :: Integer -> Integer
factorial 0 = 1
factorial n = n * factorial (n - 1)

fibonacci :: Integer -> Integer
fibonacci 0 = 0
fibonacci 1 = 1
fibonacci n = fibonacci (n - 1) + fibonacci (n - 2)
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:factorial | func:fibonacci")
        .assert()
        .success()
        .stdout(predicate::str::contains("Recursion.hs"));
}

#[test]
fn test_haskell_custom_higher_order() {
    let dir = setup_custom_project(&[(
        "HigherOrder.hs",
        r#"
module HigherOrder where

myMap :: (a -> b) -> [a] -> [b]
myMap _ [] = []
myMap f (x:xs) = f x : myMap f xs

myFilter :: (a -> Bool) -> [a] -> [a]
myFilter _ [] = []
myFilter p (x:xs)
    | p x       = x : myFilter p xs
    | otherwise = myFilter p xs

myFold :: (b -> a -> b) -> b -> [a] -> b
myFold _ acc [] = acc
myFold f acc (x:xs) = myFold f (f acc x) xs
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:myMap & func:myFilter")
        .assert()
        .success()
        .stdout(predicate::str::contains("HigherOrder.hs"));
}

#[test]
fn test_haskell_custom_list_comprehension() {
    let dir = setup_custom_project(&[(
        "Comprehensions.hs",
        r#"
module Comprehensions where

evens :: [Int] -> [Int]
evens xs = [x | x <- xs, even x]

pairs :: [a] -> [b] -> [(a, b)]
pairs xs ys = [(x, y) | x <- xs, y <- ys]

primes :: Int -> [Int]
primes n = [x | x <- [2..n], isPrime x]
  where
    isPrime k = null [d | d <- [2..k-1], k `mod` d == 0]
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:evens | func:primes")
        .assert()
        .success()
        .stdout(predicate::str::contains("Comprehensions.hs"));
}

// =============================================================================
// OUTPUT FORMAT TESTS
// =============================================================================

#[test]
fn test_haskell_format_paths() {
    let dir = setup_fixture("haskell_project");
    let output = assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--format=paths")
        .arg("func:greet")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Main.hs"));
}

#[test]
fn test_haskell_format_markdown() {
    let dir = setup_fixture("haskell_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--format=markdown")
        .arg("func:add")
        .assert()
        .success()
        .stdout(predicate::str::contains("```hs"));
}

// =============================================================================
// EDGE CASE TESTS
// =============================================================================

#[test]
fn test_haskell_not_found() {
    let dir = setup_fixture("haskell_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:nonexistent & ext:hs")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn test_haskell_ext_filter() {
    let dir = setup_fixture("haskell_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:. & ext:hs")
        .assert()
        .success()
        .stdout(predicate::str::contains(".rs").not());
}
