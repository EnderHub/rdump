package demo.app

import demo.utils.Helper

trait Greets {
  def greet(name: String): Unit
}

class Greeter extends Greets {
  def greet(name: String): Unit = {
    println(s"Hello $name")
  }
}

object Main {
  def add(a: Int, b: Int): Int = a + b

  def main(args: Array[String]): Unit = {
    val g = new Greeter()
    g.greet("world")
    println(add(1, 2))
    Helper.doNothing()
  }
}
