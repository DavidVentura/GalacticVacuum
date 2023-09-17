FROM ubuntu:16.04

RUN apt-get update && apt-get install -y curl
RUN apt-get install build-essential libasound2-dev pkg-config -y

RUN mkdir -p /src
WORKDIR /src

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
