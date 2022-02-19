package main

import (
	"encoding/json"
	"fmt"
	"net/http"

	"github.com/ccw630/clioude/hub/protocol"
	"github.com/google/uuid"
)

type Session struct {
	id string

	ioPump *IOPump

	statusPump *StatusPump

	runner *Runner

	currentStatus protocol.RunStatus

	exitCode byte
}

type SessionRequest struct {
	Language string `json:"language"`
	Code     string `json:"code"`
}

type SessionResponse struct {
	SessionId string `json:"session_id"`
}

func createSession(hub *Hub, w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	w.Header().Set("Access-Control-Allow-Origin", "*")
	w.Header().Set("Access-Control-Allow-Headers", "*")

	if r.Method == "OPTIONS" {
		w.WriteHeader(200)
		return
	}

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

			w.WriteHeader(200)

			id := uuid.New().String()
			session := &Session{
				id:     id,
				runner: runner,
			}

			hub.sessions[id] = session
			runner.session = session

			runner.send <- protocol.InitMessage(runner.id, params.Language)
			runner.send <- protocol.CodeMessage(params.Code)

			json.NewEncoder(w).Encode(SessionResponse{
				SessionId: id,
			})
			return
		}
	}

	w.WriteHeader(400)
	fmt.Fprintf(w, "No available runner")

}
