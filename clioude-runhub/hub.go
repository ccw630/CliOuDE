package main

import (
	"log"

	"github.com/google/uuid"
)

// Hub maintains the set of active clients and broadcasts messages to the
// clients.
type Hub struct {
	// Registered runners.
	runners map[*Runner]RunnerInfo

	// Inbound messages from the clients.
	broadcast chan []byte

	// Register requests from the clients.
	register chan RunnerInfo

	// Unregister requests from clients.
	unregister chan *Runner
}

type RunnerInfo struct {
	id     string
	runner *Runner
	conf   string
}

func NewHub() *Hub {
	return &Hub{
		broadcast:  make(chan []byte),
		register:   make(chan RunnerInfo),
		unregister: make(chan *Runner),
		runners:    make(map[*Runner]RunnerInfo),
	}
}

func (h *Hub) Run() {
	for {
		select {
		case runnerAndConf := <-h.register:
			id := uuid.New().String()
			runner := runnerAndConf.runner
			h.runners[runner] = runnerAndConf
			runner.send <- []byte(id + "C++\xc0")
			runner.send <- []byte("#include<iostream>\nusing namespace std;int main(){int n;cin>>n;cout<<n<<endl;}\n\xc1")
			runnerAndConf.id = id
			log.Println("Add runner to hub:", runnerAndConf)
		case runner := <-h.unregister:
			if runnerInfo, ok := h.runners[runner]; ok {
				log.Println("Remove runner to hub:", runnerInfo)
				delete(h.runners, runner)
				close(runner.send)
			}
			// case message := <-h.broadcast:
			// 	for client := range h.clients {
			// 		select {
			// 		case client.send <- message:
			// 		default:
			// 			close(client.send)
			// 			delete(h.clients, client)
			// 		}
			// 	}
		}
	}
}
