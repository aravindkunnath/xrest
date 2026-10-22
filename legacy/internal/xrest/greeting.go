package xrest

// Greeter provides greeting logic, independent of any UI or CLI framework.
type Greeter struct{}

// NewGreeter creates a new Greeter service instance.
func NewGreeter() *Greeter {
	return &Greeter{}
}

// Greet greets the given name.
func (g *Greeter) Greet(name string) string {
	if name == "" {
		return "Hello World!"
	}
	return "Hello " + name + "!"
}
