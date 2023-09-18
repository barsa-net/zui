ARG NODE_VERSION=16

FROM docker.io/library/node:$NODE_VERSION AS builder

WORKDIR /app

COPY package*.json ./
RUN npm ci

COPY . .

RUN npm run build

FROM rust:1.72 AS zwr

WORKDIR /usr/src/

COPY Cargo* ./

COPY zwr/src/main.rs zwr/src/main.rs

RUN RUSTFLAGS='-C target-feature=+crt-static' cargo build --release --target x86_64-unknown-linux-gnu

FROM gcr.io/distroless/base

WORKDIR /opt/zui

COPY --from=zwr /usr/src/target/x86_64-unknown-linux-gnu/release/zwr ./zwr
COPY --from=builder /app/build/ ./ui/

ENTRYPOINT ["/opt/zui/zwr"]