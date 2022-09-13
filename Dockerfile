# build stage
FROM rust:latest as cargo-build

WORKDIR /usr/src/verden
COPY . .

RUN cargo install --path . && cargo install sqlx-cli
EXPOSE 9090

CMD ["verden"]
