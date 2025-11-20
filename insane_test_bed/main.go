package main

import "fmt"

type Server struct {
	Address string
}

func NewServer(addr string) *Server {
	return &Server{Address: addr}
}

func main() {
	server := NewServer(":8080")
	fmt.Println(server.Address)
}
