FROM rust:1.80 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12
COPY --from=builder /app/target/release/decentralized_decision_vote /usr/local/bin/ddv
COPY --from=builder /app/config.yaml /etc/ddv/config.yaml
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/ddv", "--api-mode"]
