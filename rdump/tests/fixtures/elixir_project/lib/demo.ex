defmodule Demo do
  def greet(name) do
    IO.puts("Hello #{name}")
  end

  def add(a, b) do
    a + b
  end
end

Demo.greet("world")
IO.puts(Demo.add(1, 2))
