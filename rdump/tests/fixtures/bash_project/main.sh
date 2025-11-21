#!/usr/bin/env bash

source "$(dirname "$0")/util.sh"

greet() {
  local name="$1"
  echo "Hello ${name}"
}

add() {
  local a=$1 b=$2
  echo $((a + b))
}

greet "world"
add 1 2
do_nothing
