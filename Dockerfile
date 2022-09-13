# build stage
FROM rust:latest as cargo-build

WORKDIR /usr/src/verden
COPY . .

ARG DATABASE_URL="postgres://user:password@localhost:5432/verden"

RUN cargo install --path . && cargo install sqlx-cli
EXPOSE 9090

CMD ["verden"]
