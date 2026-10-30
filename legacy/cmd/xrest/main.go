package main

import (
	"flag"
	"fmt"
	"os"

	"xrest/internal/xrest"
)

func main() {
	nameFlag := flag.String("name", "", "Name to greet")
	shortNameFlag := flag.String("n", "", "Name to greet (shorthand)")
	
	flag.Usage = func() {
		fmt.Fprintf(os.Stderr, "Usage of %s:\n", os.Args[0])
		flag.PrintDefaults()
	}
	
	flag.Parse()

	name := *nameFlag
	if name == "" {
		name = *shortNameFlag
	}

	greeter := xrest.NewGreeter()
	fmt.Println(greeter.Greet(name))
}
