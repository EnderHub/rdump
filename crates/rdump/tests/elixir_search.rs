use predicates::prelude::*;
mod common;
use common::{setup_custom_project, setup_fixture};

// =============================================================================
// BASIC PREDICATE TESTS
// =============================================================================

#[test]
fn test_def_predicate_elixir() {
    let dir = setup_fixture("elixir_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:Demo")
        .assert()
        .success()
        .stdout(predicate::str::contains("demo.ex"));
}

#[test]
fn test_func_predicate_elixir() {
    let dir = setup_fixture("elixir_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:greet")
        .assert()
        .success()
        .stdout(predicate::str::contains("demo.ex"));
}

#[test]
fn test_call_predicate_elixir() {
    let dir = setup_fixture("elixir_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("call:greet")
        .assert()
        .success()
        .stdout(predicate::str::contains("demo.ex"));
}

#[test]
fn test_str_predicate_elixir() {
    let dir = setup_fixture("elixir_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("str:Hello")
        .assert()
        .success()
        .stdout(predicate::str::contains("demo.ex"));
}

// =============================================================================
// DEF PREDICATE TESTS
// =============================================================================

#[test]
fn test_elixir_def_demo() {
    let dir = setup_fixture("elixir_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:Demo")
        .assert()
        .success()
        .stdout(predicate::str::contains("demo.ex"));
}

// =============================================================================
// COMBINATION TESTS
// =============================================================================

#[test]
fn test_elixir_def_and_func() {
    let dir = setup_fixture("elixir_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:Demo & func:greet")
        .assert()
        .success()
        .stdout(predicate::str::contains("demo.ex"));
}

#[test]
fn test_elixir_or_operations() {
    let dir = setup_fixture("elixir_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:greet | call:greet")
        .assert()
        .success()
        .stdout(predicate::str::contains("demo.ex"));
}

// =============================================================================
// CUSTOM PROJECT TESTS
// =============================================================================

#[test]
fn test_elixir_custom_genserver() {
    let dir = setup_custom_project(&[(
        "server.ex",
        r#"
defmodule Counter do
  use GenServer

  def start_link(initial_value) do
    GenServer.start_link(__MODULE__, initial_value, name: __MODULE__)
  end

  def increment do
    GenServer.call(__MODULE__, :increment)
  end

  def init(initial_value) do
    {:ok, initial_value}
  end

  def handle_call(:increment, _from, state) do
    {:reply, state + 1, state + 1}
  end
end
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:Counter & func:start_link")
        .assert()
        .success()
        .stdout(predicate::str::contains("server.ex"));
}

#[test]
fn test_elixir_custom_struct() {
    let dir = setup_custom_project(&[(
        "user.ex",
        r#"
defmodule User do
  defstruct [:name, :email, :age]

  def new(name, email, age) do
    %User{name: name, email: email, age: age}
  end

  def adult?(%User{age: age}) do
    age >= 18
  end
end
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:User & func:new")
        .assert()
        .success()
        .stdout(predicate::str::contains("user.ex"));
}

#[test]
fn test_elixir_custom_protocol() {
    let dir = setup_custom_project(&[(
        "protocol.ex",
        r#"
defprotocol Stringify do
  def to_string(data)
end

defimpl Stringify, for: Integer do
  def to_string(data), do: Integer.to_string(data)
end

defimpl Stringify, for: List do
  def to_string(data), do: Enum.join(data, ", ")
end
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:to_string & ext:ex")
        .assert()
        .success()
        .stdout(predicate::str::contains("protocol.ex"));
}

#[test]
fn test_elixir_custom_pipeline() {
    let dir = setup_custom_project(&[(
        "pipeline.ex",
        r#"
defmodule DataProcessor do
  def process(data) do
    data
    |> validate()
    |> transform()
    |> save()
  end

  defp validate(data), do: {:ok, data}
  defp transform({:ok, data}), do: {:ok, String.upcase(data)}
  defp save({:ok, data}), do: {:saved, data}
end
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:DataProcessor & func:process")
        .assert()
        .success()
        .stdout(predicate::str::contains("pipeline.ex"));
}

#[test]
fn test_elixir_custom_pattern_matching() {
    let dir = setup_custom_project(&[(
        "patterns.ex",
        r#"
defmodule Calculator do
  def calculate({:add, a, b}), do: a + b
  def calculate({:subtract, a, b}), do: a - b
  def calculate({:multiply, a, b}), do: a * b
  def calculate({:divide, a, b}) when b != 0, do: a / b
  def calculate({:divide, _, 0}), do: {:error, "division by zero"}
end
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:Calculator & func:calculate")
        .assert()
        .success()
        .stdout(predicate::str::contains("patterns.ex"));
}

#[test]
fn test_elixir_custom_macro() {
    let dir = setup_custom_project(&[(
        "macros.ex",
        r#"
defmodule MyMacros do
  defmacro unless(condition, do: block) do
    quote do
      if !unquote(condition) do
        unquote(block)
      end
    end
  end

  defmacro debug(expr) do
    quote do
      IO.inspect(unquote(expr), label: unquote(Macro.to_string(expr)))
    end
  end
end
"#,
    )]);

    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:MyMacros & ext:ex")
        .assert()
        .success()
        .stdout(predicate::str::contains("macros.ex"));
}

// =============================================================================
// OUTPUT FORMAT TESTS
// =============================================================================

#[test]
fn test_elixir_format_paths() {
    let dir = setup_fixture("elixir_project");
    let output = assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--format=paths")
        .arg("func:greet")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("demo.ex"));
}

#[test]
fn test_elixir_format_markdown() {
    let dir = setup_fixture("elixir_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("--format=markdown")
        .arg("func:greet")
        .assert()
        .success()
        .stdout(predicate::str::contains("```ex"));
}

// =============================================================================
// EDGE CASE TESTS
// =============================================================================

#[test]
fn test_elixir_not_found() {
    let dir = setup_fixture("elixir_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("def:NonExistent & ext:ex")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn test_elixir_ext_filter() {
    let dir = setup_fixture("elixir_project");
    assert_cmd::cargo::cargo_bin_cmd!("rdump")
        .current_dir(dir.path())
        .arg("search")
        .arg("func:. & ext:ex")
        .assert()
        .success()
        .stdout(predicate::str::contains(".rs").not());
}
