FROM alpine:3.24.1

# Install system dependencies and native Alpine cargo package
RUN apk update && apk add --no-cache \
    alpine-sdk \
    cargo \
    git \
    openssl-dev


# Set the container workspace directory per your preference
WORKDIR /home/seev0/etc
