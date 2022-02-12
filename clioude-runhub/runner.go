package main

import (
	"log"
	"net/http"
	"sync"
	"time"

	"github.com/gorilla/websocket"
)

const (
	// Time allowed to write a message to the peer.
	writeWait = 10 * time.Second

	// Time allowed to read the next pong message from the peer.
	pongWait = 60 * time.Second

	// Send pings to peer with this period. Must be less than pongWait.
	pingPeriod = (pongWait * 9) / 10

	// Maximum message size allowed from peer.
	// maxMessageSize = 65536
)

var upgrader = websocket.Upgrader{
	ReadBufferSize:  1024,
	WriteBufferSize: 1024,
}

type Runner struct {
	hub *Hub

	// The websocket connection.
	conn *websocket.Conn

	// Buffered channel of outbound messages.
	send chan []byte

	id           string
	session      *Session
	buffer       chan []byte
	conf         string
	inputBarrier sync.WaitGroup
}

// readPump pumps messages from the websocket connection to the hub.
//
// The application runs readPump in a per-connection goroutine. The application
// ensures that there is at most one reader on a connection by executing all
// reads from this goroutine.
func (r *Runner) readPump() {
	defer func() {
		r.hub.unregister <- r
		r.conn.Close()
	}()
	// c.conn.SetReadLimit(maxMessageSize)
	r.conn.SetReadDeadline(time.Now().Add(pongWait))
	r.conn.SetPongHandler(func(string) error { r.conn.SetReadDeadline(time.Now().Add(pongWait)); return nil })
	_, message, err := r.conn.ReadMessage()
	log.Println("Received from runner:", message)
	if err != nil {
		if websocket.IsUnexpectedCloseError(err, websocket.CloseNoStatusReceived, websocket.CloseNormalClosure) {
			log.Printf("error on initializing runner: %v", err)
		}
		return
	}
	r.conf = string(message)
	r.hub.register <- r
	r.inputBarrier.Add(1)
	for {
		_, message, err := r.conn.ReadMessage()
		log.Println("Received from runner:", message)
		if err != nil {
			if websocket.IsUnexpectedCloseError(err, websocket.CloseNoStatusReceived, websocket.CloseNormalClosure) {
				log.Printf("error: %v", err)
			}
			break
		}
		lastByte := message[len(message)-1]
		if lastByte == 0xe7 {
			// status info
			if message[0] == 0 {
				r.inputBarrier.Done()
			}
		} else if lastByte == 0xe6 {
			// usage metrics
		} else if lastByte == 0xe8 {
			// exit info
		} else {
			if r.session.client == nil {
				r.buffer <- message
			} else {
				r.session.client.send <- message
			}
		}
	}
}

// writePump pumps messages from the hub to the websocket connection.
//
// A goroutine running writePump is started for each connection. The
// application ensures that there is at most one writer to a connection by
// executing all writes from this goroutine.
func (r *Runner) writePump() {
	ticker := time.NewTicker(pingPeriod)
	defer func() {
		ticker.Stop()
		r.conn.Close()
	}()
	for {
		select {
		case message, ok := <-r.send:
			r.conn.SetWriteDeadline(time.Now().Add(writeWait))
			if !ok {
				// The hub closed the channel.
				r.conn.WriteMessage(websocket.CloseMessage, []byte{})
				return
			}

			w, err := r.conn.NextWriter(websocket.BinaryMessage)
			if err != nil {
				return
			}
			log.Println("Send to runner:", message)
			w.Write(message)

			if err := w.Close(); err != nil {
				return
			}
		case <-ticker.C:
			r.conn.SetWriteDeadline(time.Now().Add(writeWait))
			if err := r.conn.WriteMessage(websocket.PingMessage, nil); err != nil {
				return
			}
		}
	}
}

// serveRunner handles websocket requests from the peer.
func ServeRunner(hub *Hub, w http.ResponseWriter, r *http.Request) {
	upgrader.CheckOrigin = func(r *http.Request) bool { return true }
	conn, err := upgrader.Upgrade(w, r, nil)
	if err != nil {
		log.Println(err)
		return
	}
	runner := &Runner{hub: hub, conn: conn, send: make(chan []byte, 256), buffer: make(chan []byte)}

	// Allow collection of memory referenced by the caller by doing all work in
	// new goroutines.
	go runner.writePump()
	go runner.readPump()
}
