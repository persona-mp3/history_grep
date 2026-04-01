package main

import (
	"bytes"
	"fmt"
	"io"
	"log"
	"os"
	"os/exec"
	"strings"
)

const (
	fzf        = "/opt/homebrew/bin/fzf"
	ChromePath = "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
)

func display(dump string) {
	var outBuffer bytes.Buffer

	cmd := exec.Command(fzf)
	cmd.Stdin = strings.NewReader(dump)
	cmd.Stdout = io.MultiWriter(os.Stdout, &outBuffer)

	cmd.Run()
	fmt.Printf("opening browser for: %s\n", outBuffer.String())

	chromeCmd := exec.Command(ChromePath, outBuffer.String())
	if err := chromeCmd.Run(); err != nil {
		log.Fatalf("could not open chrome. Reason: -> %s\n", err);
	}
}
