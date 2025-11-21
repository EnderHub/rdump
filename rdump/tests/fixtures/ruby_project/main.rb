require_relative "utils/helper"

module Demo
  class Greeter
    def greet(name)
      puts "Hello #{name}"
    end
  end
end

def add(a, b)
  a + b
end

g = Demo::Greeter.new
g.greet("world")
puts add(1, 2)
Demo::Helper.do_nothing
