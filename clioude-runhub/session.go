package main

import (
	"encoding/json"
	"fmt"
	"net/http"

	"github.com/google/uuid"
)

type Session struct {
	id string

	client *Client

	runner *Runner
}

type SessionRequest struct {
	Language string `json:"language"`
	Script   string `json:"script"`
}

type SessionResponse struct {
	SessionId string `json:"session_id"`
}

func createSession(hub *Hub, w http.ResponseWriter, r *http.Request) {
	if r.Method != "POST" {
		w.WriteHeader(405)
		return
	}

	decoder := json.NewDecoder(r.Body)
	var params SessionRequest
	err := decoder.Decode(&params)
	if err != nil {
		w.WriteHeader(400)
		fmt.Fprintf(w, "Invalid session request")
		return
	}

	for runner := range hub.runners {
		if runner.session == nil {

			w.Header().Set("Content-Type", "application/json")
			w.WriteHeader(200)

			id := uuid.New().String()
			session := &Session{
				id:     id,
				runner: runner,
			}

			hub.sessions[id] = session
			runner.session = session

			runner.send <- []byte(runner.id + params.Language + "\xc0")
			runner.send <- []byte(params.Script + "\xc1")

			json.NewEncoder(w).Encode(SessionResponse{
				SessionId: id,
			})
			return
		}
	}

	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(400)
	fmt.Fprintf(w, "No available runner")

}
