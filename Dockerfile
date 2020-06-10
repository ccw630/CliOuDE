FROM registry.cn-shanghai.aliyuncs.com/clioude/runenv

RUN pip3 install --no-cache-dir tornado

ADD . /app
WORKDIR /app

ENTRYPOINT /app/bin/entrypoint.sh