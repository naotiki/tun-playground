ARG BIN_NAME=server

FROM rust:1.84.0 AS build-stage
ARG BIN_NAME


ARG BUILD_MODE=debug

ARG BUILD_DIR=/app
WORKDIR ${BUILD_DIR}

RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=cache,target=${BUILD_DIR}/target/ \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    <<EOF
set -e
cargo build --locked --bin ${BIN_NAME} $(${BUILD_MODE} == "release" && echo "--release")
cp ./target/${BUILD_MODE}/${BIN_NAME} /bin/${BIN_NAME}
EOF

FROM ubuntu:24.04

ARG BIN_NAME
RUN apt update

COPY --from=build-stage /bin/${BIN_NAME} /bin/${BIN_NAME}


ENV BIN=/bin/${BIN_NAME}
CMD ${BIN}