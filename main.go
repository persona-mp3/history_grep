package main

import (
	"bytes"
	"database/sql"
	"flag"
	"fmt"
	"io"
	"log"
	"os"
	"os/exec"
	"path/filepath"
	"strings"

	_ "github.com/mattn/go-sqlite3"
)

type Url struct {
	Id          int
	LinkUrlId   int
	TopLevelUrl string
	FrameUrl    string
	VisitCount  int
}

const (
	DefaultLimit      uint64 = 600
	ChromeHistoryPath        = "/Library/Application Support/Google/Chrome/Default/History"
	ChromeExePath            = "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
)

var descriptions = map[string]string{
	"browser": `Default browser is Chrome, if browser is specified, will use it to search for history`,
	"pager":   `Default pager is fzf, if pager is specified, will use it to search for history`,
	"limit":   "Default is 600. Specifies how much recents to provide as search. Using 0 will get all of history",
}

func main() {
	var browser string
	var pager string
	var limit uint64
	flag.StringVar(&browser, "browser", "chrome", descriptions["chrome"])
	flag.StringVar(&pager, "pager", "fzf", descriptions["pager"])
	flag.Uint64Var(&limit, "limit", DefaultLimit, descriptions["limit"])

	fullPath := appendToHome(ChromeHistoryPath)
	cmd := exec.Command("cp", fullPath, "./.chrome_history")
	cmd.Stdout = os.Stdout
	if err := cmd.Run(); err != nil {
		log.Fatalf("could not copy chromes history. Reason: %s\n", err)
	}

	db, err := sql.Open("sqlite3", ".chrome_history")
	if err != nil {
		log.Fatalf("Could not open .chrome_history. Reason: %s\n", err)
	}

	defer db.Close()

	// By default we just get the last 600 link visits
	query := "select * from visited_links order by id desc limit (?)"
	rows, err := db.Query(query, limit)

	if err != nil {
		log.Fatalf("Could not query db for visited links. Reason: %s\n", err)
	}

	allUrls := []*Url{}
	for rows.Next() {
		u := &Url{}
		err := rows.Scan(&u.Id, &u.LinkUrlId, &u.TopLevelUrl, &u.FrameUrl, &u.VisitCount)
		if err != nil {
			log.Printf("Error while scanning: %s\n", err)
			continue
		}

		allUrls = append(allUrls, u)
	}

	var sb strings.Builder
	for _, url := range allUrls {
		sb.WriteString(fmt.Sprintf("%s\n", url.TopLevelUrl))
	}

	dump := sb.String()
	display(dump)
}
func getExePath(exe string) (string, error) {
	cmd := exec.Command("which", exe)
	binPath, err := cmd.Output()
	if err != nil {
		return "", err
	}

	s := strings.ToValidUTF8(string(binPath), "")
	return strings.TrimSpace(s), nil
}

func display(dump string) {
	var outBuffer bytes.Buffer

	fzfPath, err := getExePath("fzf")

	if err != nil {
		log.Fatal("Could not get fzf path. Reason: ", err)
		return
	} else if len(strings.ReplaceAll(fzfPath, " ", "")) == 0 {
		log.Fatal("Could not find path to fzf. Possibly doesn't exist, Have you installed it?")
		return
	}

	cmd := exec.Command("/opt/homebrew/bin/fzf")
	cmd.Stdin = strings.NewReader(dump)
	cmd.Stdout = io.MultiWriter(os.Stdout, &outBuffer)

	if err := cmd.Run(); err != nil && !strings.Contains(err.Error(), "130") {
		log.Fatal("Could not use fzf: ", err)
	}

	tgtUrl := outBuffer.String()
	if len(strings.ReplaceAll(tgtUrl, " ", "")) == 0 {
		return
	}

	chromeCmd := exec.Command(ChromeExePath, tgtUrl)
	if err := chromeCmd.Run(); err != nil {
		log.Fatalf("could not open chrome. Reason: -> %s\n", err)
	}
}

func appendToHome(path string) string {
	home, err := os.UserHomeDir()
	if err != nil {
		log.Fatalf("Could not find home_dir. Reason: %s\n", err)
	}
	return filepath.Join(home, path)
}
