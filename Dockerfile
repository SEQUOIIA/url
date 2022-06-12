FROM rust:1.61 as builder

RUN USER=root cargo new --bin deps-caching
WORKDIR /deps-caching
RUN USER=root cargo new --bin app
RUN USER=root cargo new --bin cli
COPY ./Cargo.toml ./Cargo.toml
COPY ./app/Cargo.toml ./app/Cargo.toml
COPY ./cli/Cargo.toml ./cli/Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
RUN cargo build --release
RUN rm -rf src && rm -rf app/src && rm -rf cli/src

COPY . .

RUN rm ./target/release/deps/seqtf_url*
RUN cargo build --bin seqtf_url --release --offline

# App assembling
FROM debian:buster-slim
ARG APP=/usr/src/app

RUN apt-get update \
    && apt-get install -y ca-certificates tzdata libsqlite3-0 \
    && rm -rf /var/lib/apt/lists/*

EXPOSE 8080

ENV TZ=Etc/UTC
ENV APP_USER=url

RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP}

COPY --from=builder /deps-caching/target/release/seqtf_url ${APP}/seqtf_url

RUN chown -R $APP_USER:$APP_USER ${APP}

USER $APP_USER
WORKDIR ${APP}

CMD ["./seqtf_url"]
