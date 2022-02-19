package protocol

import "encoding/json"

type statusMessage struct {
	Type string      `json:"type"`
	Desc interface{} `json:"desc"`
}

func ParseMessage(message []byte) (ControlFlag, []byte) {
	return ControlFlag(message[len(message)-1]), message[0 : len(message)-1]
}

func ParseStatus(status byte) []byte {
	res, _ := json.Marshal(statusMessage{Type: "status", Desc: statuses[status]})
	return res
}

func ParseExitCode(exitInfo []byte) []byte {
	res, _ := json.Marshal(statusMessage{Type: "exit", Desc: exitInfo[0]})
	return res
}

func InputMessage(input []byte) []byte {
	return append(input, 0xe0)
}

func OutputMessage(output []byte) []byte {
	return append(output, 0xe1)
}

func InitMessage(runnerId string, language string) []byte {
	// todo zero copy string->byte
	return append([]byte(runnerId+language), Init)
}

func CodeMessage(code string) []byte {
	return append([]byte(code), Code)
}

func StatusMessage(status RunStatus) []byte {
	return []byte{byte(status), StatusInfo}
}

func ExitMessage(exitCode byte) []byte {
	return []byte{exitCode, ExitInfo}
}
