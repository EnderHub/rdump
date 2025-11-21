<?php
namespace Demo\App;

use Demo\Utils\Helper;

interface Greets {
    public function greet(string $name);
}

class Greeter implements Greets {
    public function greet(string $name) {
        echo "Hello $name";
    }
}

function add(int $a, int $b) {
    return $a + $b;
}

$g = new Greeter();
$g->greet("world");
echo add(1, 2);
Helper::doNothing();
