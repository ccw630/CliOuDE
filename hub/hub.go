package main

import (
	"log"

	"github.com/ccw630/clioude/hub/protocol"
	"github.com/google/uuid"
)

// Hub maintains the set of active clients and broadcasts messages to the
// clients.
type Hub struct {
	// Activated sessions
	sessions map[string]*Session

	// Registered runners.
	runners map[*Runner]bool

	// Map language to runners
	languagedRunners map[string]map[*Runner]bool

	// Register requests from the runners.
	register chan *Runner

	// Unregister requests from runners.
	unregister chan *Runner

	// Connect requests from IO pumps.
	connect chan *IOPump

	// Disconnect requests from IO pumps.
	disconnect chan *IOPump

	// Listen requests from status pumps.
	listen chan *StatusPump

	// Unlisten requests from status pumps.
	unlisten chan *StatusPump
}

func NewHub() *Hub {
	return &Hub{
		sessions:         make(map[string]*Session),
		register:         make(chan *Runner),
		unregister:       make(chan *Runner),
		connect:          make(chan *IOPump),
		disconnect:       make(chan *IOPump),
		listen:           make(chan *StatusPump),
		unlisten:         make(chan *StatusPump),
		runners:          make(map[*Runner]bool),
		languagedRunners: make(map[string]map[*Runner]bool),
	}
}

func (h *Hub) Run() {
	for {
		select {
		case runner := <-h.register:
			id := uuid.New().String()
			runner.id = id
			h.runners[runner] = true
			for language := range runner.conf.AvailableLanguages {
				if _, ok := h.languagedRunners[language]; !ok {
					h.languagedRunners[language] = make(map[*Runner]bool)
				}
				h.languagedRunners[language][runner] = true
			}
			log.Println("Add runner to hub:", runner.id)
		case runner := <-h.unregister:
			if _, ok := h.runners[runner]; ok {
				log.Println("Remove runner to hub:", runner.id)
				if runner.session != nil {
					if runner.session.ioPump != nil {
						runner.session.ioPump.closing <- true
					}
					if runner.session.statusPump != nil {
						runner.session.statusPump.closing <- true
					}
					// TODO persist session
					// delete(h.sessions, runner.session.id)
				}
				delete(h.runners, runner)
				close(runner.send)
				for language := range runner.conf.AvailableLanguages {
					delete(h.languagedRunners[language], runner)
				}
			}
		case ioPump := <-h.connect:
			select {
			case buffer := <-ioPump.runner.buffer:
				ioPump.send <- buffer
			default:
			}
			ioPump.runner.session.ioPump = ioPump
		case ioPump := <-h.disconnect:
			close(ioPump.send)
			ioPump.runner.session.ioPump = nil
		case statusPump := <-h.listen:
			statusPump.session.statusPump = statusPump
			statusPump.send <- protocol.ParseStatus(byte(statusPump.session.CurrentStatus))
		case statusPump := <-h.unlisten:
			close(statusPump.send)
			statusPump.session.ioPump = nil
		}
	}
}
