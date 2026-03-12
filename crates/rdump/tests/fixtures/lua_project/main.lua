local util = require("util")

local function greet(name)
    print("Hello " .. name)
end

function add(a, b)
    return a + b
end

greet("world")
print(add(1, 2))
util.do_nothing()
