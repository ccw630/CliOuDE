package main

import (
	"flag"
	"net/http"
	"os"

	"go.uber.org/zap"
	"go.uber.org/zap/zapcore"
)

var addr = flag.String("addr", ":8080", "http service address")
var logger *zap.Logger
var sugar *zap.SugaredLogger

func initLogger() {
	writeSyncer := getLogWriter()
	encoder := getEncoder()
	core := zapcore.NewCore(encoder, writeSyncer, zapcore.DebugLevel)

	logger := zap.New(core, zap.AddCaller())
	sugar = logger.Sugar()
}

func getEncoder() zapcore.Encoder {
	encoderConfig := zap.NewProductionEncoderConfig()
	encoderConfig.EncodeTime = zapcore.ISO8601TimeEncoder
	encoderConfig.EncodeLevel = zapcore.CapitalLevelEncoder
	return zapcore.NewConsoleEncoder(encoderConfig)
}

func getLogWriter() zapcore.WriteSyncer {
	return zapcore.AddSync(os.Stdout)
}

func main() {
	initLogger()
	defer logger.Sync()
	flag.Parse()
	sugar.Info("Started... on", *addr)
	hub := NewHub()
	go hub.Run()
	http.HandleFunc("/endpoint-r", func(w http.ResponseWriter, r *http.Request) {
		ServeRunner(hub, w, r)
	})
	http.HandleFunc("/endpoint-io", func(w http.ResponseWriter, r *http.Request) {
		ServeClient(hub, w, r, "io")
	})
	http.HandleFunc("/endpoint-st", func(w http.ResponseWriter, r *http.Request) {
		ServeClient(hub, w, r, "status")
	})
	http.HandleFunc("/session", func(w http.ResponseWriter, r *http.Request) {
		handleSession(hub, w, r)
	})
	http.HandleFunc("/language", func(w http.ResponseWriter, r *http.Request) {
		getLanguages(hub, w, r)
	})
	err := http.ListenAndServe(*addr, nil)
	if err != nil {
		sugar.Fatal("ListenAndServe: ", err)
	}
}
