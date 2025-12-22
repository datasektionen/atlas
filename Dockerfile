ARG RUST_VERSION=1.92
ARG DEBIAN_LTS=bookworm

## BUILD
FROM rust:${RUST_VERSION}-slim-${DEBIAN_LTS} AS build

WORKDIR /atlas

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=./target \
    --mount=type=bind,source=./Cargo.toml,target=./Cargo.toml \
    --mount=type=bind,source=./Cargo.lock,target=./Cargo.lock \
    --mount=type=bind,source=./src,target=./src \
    --mount=type=bind,source=./templates,target=./templates \
    --mount=type=bind,source=./locales,target=./locales \
    \
    cargo build --locked --release \
    && cp ./target/release/atlas .

## RUN
FROM debian:${DEBIAN_LTS}-slim AS final

ARG UID=10001
ARG USER=atlas
ARG LOG_FILE=/var/log/atlas.log

RUN adduser \
    --disabled-password \
    --no-create-home \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --uid "${UID}" \
    ${USER}

RUN touch ${LOG_FILE}
RUN chown ${USER} ${LOG_FILE}
ENV ATLAS_LOG_FILE=${LOG_FILE}

USER ${USER}

WORKDIR /atlas
COPY --from=build /atlas/atlas .
COPY ./atlas.toml /atlas/atlas.toml
COPY ./splash.txt /atlas/splash.txt
COPY ./static /atlas/static

EXPOSE ${ATLAS_PORT:-6767}

ENTRYPOINT [ "./atlas" ]
