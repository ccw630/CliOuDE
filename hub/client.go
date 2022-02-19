package main

import (
	"fmt"
	"log"
	"net/http"
	"time"

	"github.com/ccw630/clioude/hub/protocol"
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

type Client struct {
	hub         *Hub
	conn        *websocket.Conn
	send        chan []byte
	closing     chan bool
	messageType int
	clientType  string
}

type IClient interface {
	readPump()
	writePump()
}

type IOPump struct {
	*Client
	runner *Runner
}

type StatusPump struct {
	*Client
	session *Session
}

func (c *Client) init() {
	c.conn.SetReadDeadline(time.Now().Add(pongWait))
	c.conn.SetPongHandler(func(string) error { c.conn.SetReadDeadline(time.Now().Add(pongWait)); return nil })
}

func (c *IOPump) readPump() {
	defer func() {
		c.hub.disconnect <- c
		c.conn.Close()
	}()
	c.init()

	c.hub.connect <- c
	for {
		_, message, err := c.conn.ReadMessage()
		log.Printf("Received from %s: %v", c.clientType, message)
		if err != nil {
			if websocket.IsUnexpectedCloseError(err, websocket.CloseNoStatusReceived, websocket.CloseNormalClosure) {
				log.Printf("error: %v", err)
			}
			break
		}
		c.runner.inputBarrier.Wait()
		c.runner.send <- protocol.InputMessage(message)
	}
}

func (c *StatusPump) readPump() {
	defer func() {
		c.hub.unlisten <- c
		c.conn.Close()
	}()
	c.init()

	c.hub.listen <- c
	for {
		_, message, err := c.conn.ReadMessage()
		log.Printf("Received from %s: %v", c.clientType, message)
		if err != nil {
			if websocket.IsUnexpectedCloseError(err, websocket.CloseNoStatusReceived, websocket.CloseNormalClosure) {
				log.Printf("error: %v", err)
			}
			break
		}
	}
}

func (c *Client) writePump() {
	ticker := time.NewTicker(pingPeriod)
	defer func() {
		ticker.Stop()
		c.conn.Close()
	}()
	for {
		select {
		case message, ok := <-c.send:
			c.conn.SetWriteDeadline(time.Now().Add(writeWait))
			if !ok {
				c.conn.WriteMessage(websocket.CloseMessage, []byte{})
				return
			}

			w, err := c.conn.NextWriter(c.messageType)
			if err != nil {
				return
			}
			log.Printf("Send to %s: %v", c.clientType, message)
			w.Write(message)

			if err := w.Close(); err != nil {
				return
			}
		case <-ticker.C:
			c.conn.SetWriteDeadline(time.Now().Add(writeWait))
			if err := c.conn.WriteMessage(websocket.PingMessage, nil); err != nil {
				return
			}
		case <-c.closing:
			return
		}
	}
}

func ServeIO(hub *Hub, w http.ResponseWriter, r *http.Request, clientType string) {
	upgrader.CheckOrigin = func(r *http.Request) bool { return true }

	query := r.URL.Query()
	sessionId := query.Get("session_id")

	session := hub.sessions[sessionId]
	if session == nil {
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(400)
		fmt.Fprintf(w, "Invalid session ID")
		return
	}

	conn, err := upgrader.Upgrade(w, r, nil)
	if err != nil {
		log.Println(err)
		return
	}
	innerClient := &Client{
		hub:         hub,
		conn:        conn,
		send:        make(chan []byte, 256),
		closing:     make(chan bool),
		messageType: websocket.TextMessage,
		clientType:  clientType,
	}
	var client IClient
	if clientType == "io" {
		client = &IOPump{
			Client: innerClient,
			runner: session.runner,
		}
	} else if clientType == "status" {
		client = &StatusPump{
			Client:  innerClient,
			session: session,
		}
	}

	go client.writePump()
	go client.readPump()
}
