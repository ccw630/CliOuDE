FROM node:12.16.3-alpine3.10

ADD . /src
WORKDIR /src

RUN yarn && yarn build

FROM nginx
COPY build/ /usr/share/nginx/html