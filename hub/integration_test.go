package main

import (
	"bytes"
	"encoding/json"
	"io/ioutil"
	"net/http"
	"net/url"
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/ccw630/clioude/hub/protocol"
	"github.com/gorilla/websocket"
)

func mockRunner(t *testing.T, runnerReady *sync.WaitGroup) {
	u := url.URL{Scheme: "ws", Host: "localhost:8080", Path: "/endpoint-r"}
	c, _, err := websocket.DefaultDialer.Dial(u.String(), nil)
	if err != nil {
		t.Fatal("Runner dial:", err)
	}
	defer c.Close()
	c.WriteMessage(websocket.BinaryMessage, []byte("{\"available_languages\": {\"echo\":\"\"}}"))
	runnerReady.Done()
	c.ReadMessage() // id&language
	c.ReadMessage() // code
	c.WriteMessage(websocket.BinaryMessage, protocol.RunStatusMessage(protocol.Preparing))
	time.Sleep(500 * time.Millisecond)
	c.WriteMessage(websocket.BinaryMessage, protocol.RunStatusMessage(protocol.Running))
	_, message, err := c.ReadMessage()
	if err != nil {
		t.Fatal("Runner read message fail:", err)
	}
	c.WriteMessage(websocket.BinaryMessage, protocol.OutputMessage(message))
	c.WriteMessage(websocket.BinaryMessage, protocol.RunStatusMessage(protocol.Ok))
	c.WriteMessage(websocket.BinaryMessage, protocol.ExitMessage(0))
	c.WriteMessage(websocket.CloseMessage, nil)
}

func createSessionForTest(t *testing.T, runnerReady *sync.WaitGroup) *SessionResponse {
	var sessionReq = []byte(`{
		"language": "echo",
		"code": "wait 0.5s at prepare"
	}`)
	request, err := http.NewRequest("POST", "http://localhost:8080/session", bytes.NewBuffer(sessionReq))
	request.Header.Set("Content-Type", "application/json")
	if err != nil {
		t.Fatal("POST Session make req fail:", err)
	}

	runnerReady.Wait()
	client := &http.Client{}
	response, err := client.Do(request)
	if err != nil {
		t.Fatal("POST Session fail:", err)
	}
	defer response.Body.Close()

	if response.Status != "200 OK" {
		t.Fatal("When creating session, runner assign failed!")
	}
	body, _ := ioutil.ReadAll(response.Body)
	var sessionRes SessionResponse
	if err := json.Unmarshal(body, &sessionRes); err != nil {
		t.Fatal("Session response got, but unmarshal failed")
	}
	return &sessionRes
}

func mockIOPump(t *testing.T, sessionRes *SessionResponse) {
	u := url.URL{Scheme: "ws", Host: "localhost:8080", Path: "/endpoint-io", RawQuery: "session_id=" + sessionRes.SessionId}
	c, _, err := websocket.DefaultDialer.Dial(u.String(), nil)
	if err != nil {
		t.Fatal("Client dial:", err)
	}
	defer c.Close()
	input := []byte("123\n")
	c.WriteMessage(websocket.TextMessage, input)
	_, message, err := c.ReadMessage()
	if err != nil {
		t.Fatal("Client read message fail:", err)
	}
	_, output := protocol.ParseMessage(message)
	if !bytes.Equal(input, output) {
		t.Fatalf("Echo mismatch, send %s but received %s", input, message)
	}
	c.WriteMessage(websocket.CloseMessage, nil)
}

func mockStatusPump(t *testing.T, sessionRes *SessionResponse) {
	u := url.URL{Scheme: "ws", Host: "localhost:8080", Path: "/endpoint-st", RawQuery: "session_id=" + sessionRes.SessionId}
	c, _, err := websocket.DefaultDialer.Dial(u.String(), nil)
	if err != nil {
		t.Fatal("Client dial:", err)
	}
	defer c.Close()
	for {
		_, message, err := c.ReadMessage()
		if err != nil {
			t.Fatal("Client read message fail:", err)
		}
		t.Logf("status %s", message)
		if strings.Contains(string(message), "exit") {
			break
		}
	}
	c.WriteMessage(websocket.CloseMessage, nil)
}

func TestMain(t *testing.T) {
	var barrier sync.WaitGroup
	var runnerReady sync.WaitGroup
	barrier.Add(3)
	runnerReady.Add(1)
	go func() {
		main()
		barrier.Done()
	}()
	go func() {
		t.Log("Runner start")
		mockRunner(t, &runnerReady)
		t.Log("Runner done")
		barrier.Done()
	}()
	sessionRes := createSessionForTest(t, &runnerReady)
	go func() {
		t.Log("Status start")
		mockStatusPump(t, sessionRes)
		t.Log("Status done")
		barrier.Done()
	}()
	go func() {
		t.Log("IO start")
		mockIOPump(t, sessionRes)
		t.Log("IO done")
		barrier.Done()
	}()
	barrier.Wait()
}
