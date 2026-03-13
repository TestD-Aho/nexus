FROM rust:1.75-bookworm as builder

WORKDIR /app
COPY Cargo.toml ./
COPY src ./src

RUN apt-get update && apt-get install -y libssl-dev pkg-config
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 ca-certificates

WORKDIR /app
COPY --from=builder /app/target/release/nexus-server .

ENV DATABASE_URL=postgres://nexus:nexus_secret@localhost:5432/nexus
ENV JWT_SECRET=nexus-secret
ENV HOST=0.0.0.0
ENV PORT=3000

EXPOSE 3000
CMD ["./nexus-server"]
