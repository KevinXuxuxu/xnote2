FROM ubuntu:22.04

COPY target/release/xnote /app/
COPY static/ /app/static/

WORKDIR /app

ENTRYPOINT [ "./xnote" ]