package main

import "xrest/internal/xrest"

// GreetService is the Wails service adapter.
type GreetService struct {
	greeter *xrest.Greeter
}

// NewGreetService creates a new GreetService adapter.
func NewGreetService(greeter *xrest.Greeter) *GreetService {
	return &GreetService{greeter: greeter}
}

// Greet wraps the core greeting logic for Wails bindings.
func (g *GreetService) Greet(name string) string {
	return g.greeter.Greet(name)
}
