import Foundation

protocol Greeter {
    func greet(_ name: String)
}

struct Point {
    let x: Int
    let y: Int
}

class ConsoleGreeter: Greeter {
    func greet(_ name: String) {
        print("Hello \(name)")
    }
}

func add(_ a: Int, _ b: Int) -> Int {
    return a + b
}

func main() {
    let g: Greeter = ConsoleGreeter()
    g.greet("world")
    let p = Point(x: 1, y: 2)
    print(add(p.x, p.y))
}

main()
