#!/bin/bash
rm -rf /worker/*
mkdir -p /worker/run /worker/socks

chown compiler:code /worker/run
chmod 711 /worker/run

core=$(grep --count ^processor /proc/cpuinfo)
n=$(($core*2))
exec gunicorn --workers $n --threads $n --error-logfile /log/gunicorn.log --time 600 --bind 0.0.0.0:8080 server:app
