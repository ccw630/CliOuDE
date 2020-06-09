FROM registry.cn-shanghai.aliyuncs.com/clioude/runenv

COPY kernel /tmp/kernel

RUN pip3 install --no-cache-dir psutil tornado aiofile requests && \
    cd /tmp/kernel && mkdir build && cd build && cmake .. && make && make install && \
    apt-get purge -y --auto-remove $buildDeps && \
    apt-get clean && rm -rf /var/lib/apt/lists/* && \
    mkdir -p /code && \
    useradd -u 12001 compiler && useradd -u 12002 code

HEALTHCHECK --interval=5s --retries=3 CMD python3 /code/service.py
ADD server /code
WORKDIR /code
RUN gcc -shared -fPIC -o unbuffer.so unbuffer.c
EXPOSE 8080
ENTRYPOINT /code/entrypoint.sh
