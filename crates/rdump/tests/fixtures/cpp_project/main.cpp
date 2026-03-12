#include "util.hpp"
#include <iostream>

namespace demo {
struct Point {
    int x;
    int y;
};

class Greeter {
public:
    Greeter() = default;
    void greet(const std::string& name) {
        std::cout << "Hello " << name << std::endl;
    }
};

int add(int a, int b) {
    return a + b;
}
} // namespace demo

int main() {
    demo::Greeter g;
    g.greet("world");
    return demo::add(1, 2);
}
