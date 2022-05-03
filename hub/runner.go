package main

import (
	"encoding/json"
	"net/http"
	"sync"

	"github.com/ccw630/clioude/hub/protocol"
	"github.com/gorilla/websocket"
)

type Runner struct {
	*Client
	session      *Session
	buffer       chan []byte
	id           string
	conf         *RunnerConf
	inputBarrier sync.WaitGroup
}

type RunnerConf struct {
	CpuFrequency       uint64            `json:"cpu_freq"`
	TotalMemory        uint64            `json:"total_memory"`
	AvailableLanguages map[string]string `json:"available_languages"`
}

func max(x, y uint64) uint64 {
	if x > y {
		return x
	}
	return y
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
	r.init()
	_, message, err := r.conn.ReadMessage()
	sugar.Debug("Received from runner:", message)
	if err != nil {
		if websocket.IsUnexpectedCloseError(err, websocket.CloseNoStatusReceived, websocket.CloseNormalClosure) {
			sugar.Warnf("error on initializing runner: %v", err)
		}
		return
	}
	if err := json.Unmarshal(message, &r.conf); err != nil {
		sugar.Warnf("error on unmarshaling runner conf: %v", err)
		return
	}
	sugar.Debug("Received runner config:", r.conf)
	r.hub.register <- r
	r.inputBarrier.Add(1)
	for {
		_, message, err := r.conn.ReadMessage()
		sugar.Debug("Received from runner:", message)
		if err != nil {
			if websocket.IsUnexpectedCloseError(err, websocket.CloseNoStatusReceived, websocket.CloseNormalClosure) {
				sugar.Warnf("error: %v", err)
			}
			break
		}
		flag, message := protocol.ParseMessage(message)
		if flag == protocol.StatusInfo {
			if message[0] == byte(protocol.Running) {
				r.inputBarrier.Done()
			}
			status := message[0]
			r.session.CurrentStatus = protocol.RunStatus(status)
			if r.session.statusPump != nil {
				r.session.statusPump.send <- protocol.ParseStatus(status)
			}
		} else if flag == protocol.UsageInfo {
			// usage metrics
			usage := protocol.ParseUsage(message)
			sugar.Debugf("Received usage info:", usage)
			r.session.CpuTimesEstimate += float64(r.conf.CpuFrequency) * float64(r.session.lastCpuPercent+usage.CpuPercent) * (usage.Time - r.session.TimeElapsed) / 200
			r.session.TimeElapsed = usage.Time
			r.session.MemoryMaxUsed = max(r.session.MemoryMaxUsed, usage.Memory)
			r.session.lastCpuPercent = usage.CpuPercent
		} else if flag == protocol.ExitInfo {
			r.session.ExitCode = message[0]
			if r.session.statusPump != nil {
				r.session.statusPump.send <- protocol.ParseExitCode(message)
			}
		} else {
			if r.session.ioPump == nil {
				r.buffer <- message
			} else {
				r.session.ioPump.send <- message
			}
		}
	}
}

// serveRunner handles websocket requests from the peer.
func ServeRunner(hub *Hub, w http.ResponseWriter, r *http.Request) {
	upgrader.CheckOrigin = func(r *http.Request) bool { return true }
	conn, err := upgrader.Upgrade(w, r, nil)
	if err != nil {
		sugar.Error(err)
		return
	}
	runner := &Runner{
		Client: &Client{
			hub:         hub,
			conn:        conn,
			send:        make(chan []byte, 256),
			closing:     make(chan bool),
			messageType: websocket.BinaryMessage,
			clientType:  "runner",
		},
		buffer: make(chan []byte),
	}

	// Allow collection of memory referenced by the caller by doing all work in
	// new goroutines.
	go runner.writePump()
	go runner.readPump()
}
