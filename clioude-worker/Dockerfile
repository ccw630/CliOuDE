FROM ccw630/clioude-runenv

COPY kernel /tmp/kernel

RUN pip3 install --no-cache-dir psutil tornado requests && \
    cd /tmp/kernel && mkdir build && cd build && cmake .. && make && make install && \
    apt-get purge -y --auto-remove $buildDeps && \
    apt-get clean && rm -rf /var/lib/apt/lists/* && \
    mkdir -p /code && \
    useradd -u 12001 compiler && useradd -u 12002 code

ADD server /code
WORKDIR /code
RUN gcc -shared -fPIC -o unbuffer.so unbuffer.c
ENTRYPOINT /code/entrypoint.sh
