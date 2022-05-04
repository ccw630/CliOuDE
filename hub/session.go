package main

import (
	"encoding/json"
	"fmt"
	"net/http"

	"github.com/ccw630/clioude/hub/protocol"
	"github.com/google/uuid"
)

type Session struct {
	Id string `json:"id"`

	ioPump *IOPump

	statusPump *StatusPump

	runner *Runner

	CurrentStatus    protocol.RunStatus `json:"current_status"`
	ExitCode         byte               `json:"exit_code"`
	TimeElapsed      float64            `json:"time_elapsed"`
	CpuTimesEstimate float64            `json:"cpu_times_estimate"`
	MemoryMaxUsed    uint64             `json:"memory_max_used"`
	lastCpuPercent   float32
}

type SessionRequest struct {
	Language string `json:"language"`
	Code     string `json:"code"`
}

type SessionResponse struct {
	SessionId string `json:"session_id"`
}

func handleSession(hub *Hub, w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	w.Header().Set("Access-Control-Allow-Origin", "*")
	w.Header().Set("Access-Control-Allow-Headers", "*")

	if r.Method == "OPTIONS" {
		w.WriteHeader(200)
		return
	}

	if r.Method == "GET" {
		query := r.URL.Query()
		sessionId := query.Get("id")

		session := hub.sessions[sessionId]
		if session == nil {
			w.Header().Set("Content-Type", "application/json")
			w.WriteHeader(400)
			fmt.Fprintf(w, "Invalid session ID")
			return
		}

		w.WriteHeader(200)
		json.NewEncoder(w).Encode(session)
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

	languagedRunners, ok := hub.languagedRunners[params.Language]
	if !ok {
		w.WriteHeader(400)
		fmt.Fprintf(w, "Language not supported")
		return
	}

	for runner := range languagedRunners {
		if runner.session == nil {

			w.WriteHeader(200)

			id := uuid.New().String()
			session := &Session{
				Id:               id,
				runner:           runner,
				ExitCode:         255,
				CpuTimesEstimate: 0,
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
