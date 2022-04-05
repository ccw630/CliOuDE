package main

import (
	"encoding/json"
	"net/http"
)

func getLanguages(hub *Hub, w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	w.Header().Set("Access-Control-Allow-Origin", "*")
	w.Header().Set("Access-Control-Allow-Headers", "*")

	if r.Method != "GET" {
		w.WriteHeader(405)
		return
	}

	languages := make([]string, 0, len(hub.languagedRunners))
	for language, runners := range hub.languagedRunners {
		if len(runners) > 0 {
			languages = append(languages, language)
		}
	}

	w.WriteHeader(200)
	json.NewEncoder(w).Encode(languages)

}
