package main

import (
	"flag"
	"log"
	"net/http"
)

var addr = flag.String("addr", ":8080", "http service address")

func main() {
	flag.Parse()
	log.Println("Started... on", *addr)
	hub := NewHub()
	go hub.Run()
	http.HandleFunc("/endpoint-r", func(w http.ResponseWriter, r *http.Request) {
		ServeRunner(hub, w, r)
	})
	http.HandleFunc("/endpoint-io", func(w http.ResponseWriter, r *http.Request) {
		ServeClient(hub, w, r, "io")
	})
	http.HandleFunc("/endpoint-st", func(w http.ResponseWriter, r *http.Request) {
		ServeClient(hub, w, r, "status")
	})
	http.HandleFunc("/session", func(w http.ResponseWriter, r *http.Request) {
		createSession(hub, w, r)
	})
	http.HandleFunc("/language", func(w http.ResponseWriter, r *http.Request) {
		getLanguages(hub, w, r)
	})
	err := http.ListenAndServe(*addr, nil)
	if err != nil {
		log.Fatal("ListenAndServe: ", err)
	}
}
