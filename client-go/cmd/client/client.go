package main

import (
	"client/internal/client"
	"fmt"
)

func main() {
	fmt.Printf("In main\n")
	client := client.NewClient()

	endpoint := "127.0.0.1:4444"
	if err := client.Connect(endpoint); err != nil {
		fmt.Printf("Error during Connect: %s\n", err)
		panic(err)
	}

	fmt.Printf("Loading survey\n")
	moduleId, err := client.Load("survey")
	if err != nil {
		fmt.Printf("Error during Load: %s\n", err)
		panic(err)
	}
	fmt.Printf("Loaded module. Id: %d\n", moduleId)

	client.Close()
}
