FROM rust:1.95.0

WORKDIR /root

COPY ./migrations .

RUN cargo install sqlx-cli --no-default-features --features=postgres,native-tls

CMD [ "sqlx", "migrate", "run" ]