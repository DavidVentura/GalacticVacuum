FROM ubuntu:18.04

RUN apt-get update && apt-get install -y curl
RUN apt-get install build-essential -y

RUN mkdir -p /src
WORKDIR /src

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN apt-get install libasound2-dev -y
RUN apt-get install pkg-config -y
