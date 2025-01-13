package main

import (
	"circuitbreaker/cmd/vis/app"
	"fmt"
	"os"
)

func main() {
	if err := app.Cmd().Execute(); err != nil {
		fmt.Println(err)
		os.Exit(1)
	}
}
