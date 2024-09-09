package main

import (
	"os"
	"strings"
	"time"

	"github.com/yuin/goldmark"
)

func Must[T any](t T, err error) T {
	if err != nil {
		panic(err)
	}
	return t
}

func main() {
	filename := os.Args[1]
	s := time.Now()
	buf := &strings.Builder{}
	file := Must(os.ReadFile(filename))
	for range 1000 {
		err := goldmark.Convert(file, buf)
		if err != nil {
			panic(err)
		}
	}
	println(time.Since(s).String())
}
