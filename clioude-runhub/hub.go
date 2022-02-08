package main

import (
	"log"

	"github.com/google/uuid"
)

// Hub maintains the set of active clients and broadcasts messages to the
// clients.
type Hub struct {
	// Registered runners.
	runners map[*Runner]bool

	// Register requests from the runners.
	register chan *Runner

	// Unregister requests from runners.
	unregister chan *Runner

	// Connect requests from clients.
	connect chan *Client

	// Disconnect requests from clients.
	disconnect chan *Client
}

func NewHub() *Hub {
	return &Hub{
		register:   make(chan *Runner),
		unregister: make(chan *Runner),
		connect:    make(chan *Client),
		disconnect: make(chan *Client),
		runners:    make(map[*Runner]bool),
	}
}

func (h *Hub) Run() {
	for {
		select {
		case runner := <-h.register:
			id := uuid.New().String()
			runner.send <- []byte(id + "C++\xc0")
			runner.send <- []byte("#include<iostream>\nusing namespace std;int main(){int n;cin>>n;cout<<n<<endl;}\n\xc1")
			runner.id = id
			h.runners[runner] = true
			log.Println("Add runner to hub:", runner)
		case runner := <-h.unregister:
			if _, ok := h.runners[runner]; ok {
				log.Println("Remove runner to hub:", runner)
				delete(h.runners, runner)
				close(runner.send)
			}
		case client := <-h.connect:
			log.Println("Connection from client:", client)
			for runner := range h.runners {
				if runner.ready && runner.client == nil {
					log.Println("Find runner for client:", runner.id)
					client.runner = runner
					select {
					case buffer := <-runner.buffer:
						client.send <- buffer
					default:
					}
					runner.client = client
					break
				}
			}
		case client := <-h.disconnect:
			log.Println("Client disconnect:", client)
			client.runner.client = nil
			client.runner = nil
		}
	}
}
