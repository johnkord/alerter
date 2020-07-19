FROM ubuntu:20.04

RUN apt-get update && apt-get install libsqlite3-0

ADD "https://www.random.org/cgi-bin/randbyte?nbytes=10&format=h" skipcache
COPY target/debug/simple_alerts_backend /
COPY web /web
COPY Rocket.toml /
CMD ["/simple_alerts_backend"]
EXPOSE 8000
