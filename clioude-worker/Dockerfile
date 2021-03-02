FROM ccw630/clioude-kernel

RUN pip3 install --no-cache-dir psutil tornado requests && \
    mkdir -p /code && \
    useradd -u 12001 compiler && useradd -u 12002 code

ADD worker /code
WORKDIR /code
RUN cp /kernel/unbuffer.so /code && \
    rm -rf /kernel
ENTRYPOINT /code/entrypoint.sh
