 CREATE TABLE urls(
     id INTEGER PRIMARY KEY AUTOINCREMENT,
     url LONGVARCHAR,
     title LONGVARCHAR,
     visit_count INTEGER DEFAULT 0 NOT NULL,
     typed_count INTEGER DEFAULT 0 NOT NULL,
     last_visit_time INTEGER NOT NULL,
     hidden INTEGER DEFAULT 0 NOT NULL
 );

 CREATE INDEX urls_url_index ON urls (url);



CREATE TABLE visited_links(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    link_url_id INTEGER NOT NULL,
    top_level_url LONGVARCHAR NOT NULL,
    frame_url LONGVARCHAR NOT NULL,
    visit_count INTEGER DEFAULT 0 NOT NULL
);
CREATE INDEX visited_links_index ON visited_links (link_url_id, top_level_url, frame_url);
