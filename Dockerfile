FROM node:22 AS nodebuilder
WORKDIR /usr/src/app/
COPY frontend . 
RUN npm i && npm run build

FROM rust:bookworm AS builder
WORKDIR /usr/src/app
COPY . . 
COPY .sqlx .sqlx
RUN cargo build --release



FROM debian:bookworm-slim
RUN apt-get update && apt install -y openssl
RUN apt-get update && apt install -y openssl ca-certificates && update-ca-certificates

WORKDIR /usr/src/app
COPY --from=builder /usr/src/app/config.json /usr/src/app/
COPY --from=builder /usr/src/app/target/release/mithrilforge_server /usr/src/app/
COPY --from=nodebuilder /usr/src/app/dist/* /usr/src/app/static/
CMD ["./mithrilforge_server"]
EXPOSE 5558
