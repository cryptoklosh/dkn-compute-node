FROM rust:1.83 AS builder

# https://docs.docker.com/engine/reference/builder/#automatic-platform-args-in-the-global-scope
#
# We use distroless, which allow the following platforms:
#   linux/amd64
#   linux/arm64
#   linux/arm
#
# To build an image & push them to Docker hub for this Dockerfile:
#
# docker buildx build --platform=linux/amd64,linux/arm64,linux/arm . -t firstbatch/dria-compute-node:latest --builder=dria-builder --push   

# build release binary
WORKDIR /usr/src/app
COPY . .
RUN cargo build --bin dkn-compute --release

# copy release binary to distroless
FROM debian:12
WORKDIR /root
RUN apt-get update && apt-get install -y curl ca-certificates 
COPY --from=builder /usr/src/app/target/release/dkn-compute /root/dkn-compute

COPY entrypoint.sh /root/entrypoint.sh
RUN chmod +x /root/entrypoint.sh
ENTRYPOINT [ "/root/entrypoint.sh" ]

EXPOSE 8080

CMD ["/root/dkn-compute"]
