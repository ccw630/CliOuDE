package main

import (
	"flag"
	"log"
	"net/http"

	ws "github.com/ccw630/clioude/clioude-runhub/pkg/ws"
)

var addr = flag.String("addr", ":8080", "http service address")

func main() {
	log.Println("Started...")
	flag.Parse()
	hub := ws.NewHub()
	go hub.Run()
	http.HandleFunc("/ws", func(w http.ResponseWriter, r *http.Request) {
		ws.ServeWs(hub, w, r)
	})
	err := http.ListenAndServe(*addr, nil)
	if err != nil {
		log.Fatal("ListenAndServe: ", err)
	}
}
