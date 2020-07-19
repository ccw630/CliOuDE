FROM ubuntu:20.04
ENV DEBIAN_FRONTEND noninteractive

COPY build/java_policy /etc
COPY sdkman /opt/sdkman
COPY jdt-language-server /opt/jdt-ls
COPY kt-ls /opt/kt-ls

SHELL ["/bin/bash", "-c"]

RUN buildDeps='software-properties-common git libtool cmake python-dev python3-pip libseccomp-dev wget curl zip' && \
    apt-get update && apt-get install -y python python3 python-pkg-resources python3-pkg-resources gcc g++ socat openjdk-11-jdk vim strace ccls $buildDeps

RUN mkdir -p /etc/nodejs && cd /etc/nodejs && \
	curl -sSL https://nodejs.org/dist/latest-v12.x/node-v12.18.0-linux-x64.tar.xz | tar x --xz --strip-components=1 && \
	ln -s /etc/nodejs/bin/node /usr/bin/node && \
    /etc/nodejs/bin/npm i -g bash-language-server

RUN	ln -s /opt/sdkman/kotlin/1.3.50/bin/kotlinc /usr/bin/kotlinc && \
    ln -s /opt/sdkman/scala/2.13.0/bin/scalac /usr/bin/scalac && \
    cp -r /opt/sdkman/kotlin/1.3.50/lib /usr/lib/kotlin && \
    cp -r /opt/sdkman/scala/2.13.0/lib /usr/lib/scala

RUN	cd /tmp && git clone --depth 1 https://github.com/pocmo/Python-Brainfuck.git && mv Python-Brainfuck /usr/bin/brainfuck

RUN pip3 install --no-cache-dir python-language-server
