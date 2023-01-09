FROM rust:1.66 AS builder
WORKDIR /app
COPY . .
RUN cargo b --release

FROM debian:stable-slim
COPY --from=builder /app/target/release/mcuptime-bot /bin/mcuptime-bot
CMD /bin/mcuptime-bot