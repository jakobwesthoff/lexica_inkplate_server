FROM rust:1.64.0-buster as builder

# Trick to cache all used dependencies
RUN USER=root cargo new --bin lexica_inkplate_server
WORKDIR /lexica_inkplate_server
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release
RUN rm src/*.rs

ADD . ./

RUN rm ./target/release/deps/lexica_inkplate_server*
RUN cargo build --release

FROM debian:buster-slim
ARG APP=/app

RUN apt-get update \
    && apt-get install -y tini ca-certificates tzdata libcurl4 \
    && rm -rf /var/lib/apt/lists/*

ENV TZ=Etc/UTC \
    APP_USER=appuser \
    ROCKET_ADDRESS="0.0.0.0"

RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP}

COPY --from=builder /lexica_inkplate_server/target/release/lexica_inkplate_server ${APP}/lexica_inkplate_server

RUN chown -R $APP_USER:$APP_USER ${APP}

USER $APP_USER
WORKDIR ${APP}

EXPOSE 8000
ENTRYPOINT [ "/usr/bin/tini", "--" ]
CMD ["/app/lexica_inkplate_server"]
