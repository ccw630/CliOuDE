package main

import (
	"log"
	"net/http"
	"time"

	"github.com/gorilla/websocket"
)

type Client struct {
	hub *Hub

	conn *websocket.Conn

	send chan []byte

	runner *Runner
}

func (c *Client) readPump() {
	defer func() {
		c.hub.disconnect <- c
		c.conn.Close()
	}()
	c.conn.SetReadDeadline(time.Now().Add(pongWait))
	c.conn.SetPongHandler(func(string) error { c.conn.SetReadDeadline(time.Now().Add(pongWait)); return nil })

	c.hub.connect <- c
	for {
		_, message, err := c.conn.ReadMessage()
		log.Println("Received from client:", message)
		if err != nil {
			if websocket.IsUnexpectedCloseError(err, websocket.CloseNoStatusReceived, websocket.CloseNormalClosure) {
				log.Printf("error: %v", err)
			}
			break
		}
		c.runner.send <- message
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

			w, err := c.conn.NextWriter(websocket.BinaryMessage)
			if err != nil {
				return
			}
			log.Println("Send to client:", message)
			w.Write(message)

			if err := w.Close(); err != nil {
				return
			}
		case <-ticker.C:
			c.conn.SetWriteDeadline(time.Now().Add(writeWait))
			if err := c.conn.WriteMessage(websocket.PingMessage, nil); err != nil {
				return
			}
		}
	}
}

func ServeClient(hub *Hub, w http.ResponseWriter, r *http.Request) {
	upgrader.CheckOrigin = func(r *http.Request) bool { return true }
	conn, err := upgrader.Upgrade(w, r, nil)
	if err != nil {
		log.Println(err)
		return
	}
	client := &Client{hub: hub, conn: conn, send: make(chan []byte, 256)}

	go client.writePump()
	go client.readPump()
}
