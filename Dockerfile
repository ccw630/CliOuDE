FROM registry.cn-shanghai.aliyuncs.com/clioude/runenv

RUN pip3 install --no-cache-dir tornado

ADD . /app
WORKDIR /app

RUN mkdir -p /tmp/ls && touch /tmp/ls/Main.java

ENTRYPOINT /app/bin/entrypoint.sh