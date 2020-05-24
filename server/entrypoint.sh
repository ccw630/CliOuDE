#!/bin/bash
rm -rf /worker/*
mkdir -p /worker/run /worker/socks /log

chown compiler:code /worker/run
chmod 711 /worker/run

exec python3 server.py
