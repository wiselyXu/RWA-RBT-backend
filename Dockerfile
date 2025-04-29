FROM ubuntu:latest AS server
#FROM m.daocloud.io/docker.io/library/ubuntu:latest AS server
#RUN apt-get update && apt-get install -y openssl && rm -rf /var/lib/apt/LISTS/*
#FROM alpine:latest AS opc-mqtt
ENV TZ=Asia/Shanghai
#RUN ln -snf /usr/share/zoneinfo/$TimeZone /etc/localtime && echo $TimeZone > /etc/timezone
WORKDIR /app
COPY   ./target/x86_64-unknown-linux-musl/release/api-server ./
ENTRYPOINT ./api-server -e prod

