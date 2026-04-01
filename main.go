package main

import (
	"database/sql"
	"fmt"
	"log"
	"strings"

	_ "github.com/mattn/go-sqlite3"
)


type entry struct {
	id            int
	link_url_id   int
	top_level_url string
	frame_url     string
	visit_count   int
}

func main() {
	db, err := sql.Open("sqlite3", "./chrome_history")
	if err != nil {
		log.Fatal("couldn't open sql file -> ", err)
	}

	defer db.Close()

	rows, err := db.Query("select * from visited_links order by id desc limit 500")
	if err != nil {
		log.Fatal("query_error -> ", err)
	}

	var all_entries = []*entry{}
	for rows.Next() {
		e := &entry{}
		err := rows.Scan(&e.id, &e.link_url_id, &e.top_level_url, &e.frame_url, &e.visit_count)
		all_entries = append(all_entries, e)
		if err != nil {
			log.Printf("err scanning row -> %s\n", err)
			continue
		}

	}

	var s strings.Builder
	for _, e := range all_entries {
		s.WriteString(fmt.Sprintf("%s\n", e.top_level_url))
	}

	display(s.String())
}
