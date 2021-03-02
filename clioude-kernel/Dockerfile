FROM ccw630/clioude-runenv

COPY . /kernel

RUN cd /kernel && mkdir build && cd build && cmake .. && make && make install && \
    apt-get clean && rm -rf /var/lib/apt/lists/* && \
    cd /kernel && gcc -shared -fPIC -o unbuffer.so unbuffer.c
