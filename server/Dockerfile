FROM ${ARCH}debian:bullseye AS base
#
# Install Rust
RUN apt-get update && apt-get install -y curl
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain nightly

ENV PATH=/root/.cargo/bin:$PATH

# Switch Rust to the nightly bui
RUN rustup toolchain install nightly
RUN rustup install nightly

RUN apt-get install -y build-essential

FROM base as builder
WORKDIR /source
RUN apt-get install -y cmake
ADD . ./
ENV CARGO_UNSTABLE_SPARSE_REGISTRY=true
RUN cargo build --release

FROM ${ARCH}debian:bullseye-slim
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

COPY --from=builder /source/target/release/lexica_inkplate_server ${APP}/lexica_inkplate_server

RUN chown -R $APP_USER:$APP_USER ${APP}

USER $APP_USER
WORKDIR ${APP}

EXPOSE 8000
ENTRYPOINT [ "/usr/bin/tini", "--" ]
CMD ["/app/lexica_inkplate_server"]
