package main

import (
	"log"

	"github.com/google/uuid"
)

// Hub maintains the set of active clients and broadcasts messages to the
// clients.
type Hub struct {
	// Activated sessions
	sessions map[string]*Session

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
		sessions:   make(map[string]*Session),
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
			runner.id = id
			h.runners[runner] = true
			log.Println("Add runner to hub:", runner.id)
		case runner := <-h.unregister:
			if _, ok := h.runners[runner]; ok {
				log.Println("Remove runner to hub:", runner.id)
				if runner.session != nil && runner.session.client != nil {
					runner.session.client.closing <- true
				}
				delete(h.runners, runner)
				close(runner.send)
			}
		case client := <-h.connect:
			session := client.session
			select {
			case buffer := <-session.runner.buffer:
				client.send <- buffer
			default:
			}
			session.client = client
		case client := <-h.disconnect:
			close(client.send)
			session := client.session
			delete(h.sessions, session.id)
			session.runner.session = nil
		}
	}
}
