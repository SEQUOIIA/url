ARG BASE_IMAGE=rust:1.61.0-slim-buster

FROM $BASE_IMAGE as planner
WORKDIR app
RUN cargo install cargo-chef --version 0.1.35
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM $BASE_IMAGE as cacher
WORKDIR app
RUN cargo install cargo-chef --version 0.1.35
RUN apt update && apt install -y ca-certificates wget gcc libssl-dev libc6-dev pkg-config libsqlite3-0 libsqlite3-dev
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

FROM $BASE_IMAGE as builder
WORKDIR app
RUN apt update && apt install -y ca-certificates wget gcc libssl-dev libc6-dev pkg-config libsqlite3-0 libsqlite3-dev
COPY . .
# Copy over the cached dependencies
COPY --from=cacher /app/target target
COPY --from=cacher $CARGO_HOME $CARGO_HOME
RUN cargo build --bin seqtf_url --release

FROM $BASE_IMAGE as runtime-deps
RUN cd /tmp && \
    apt-get update && \
    apt-get install -y --no-install-recommends \
        # install only deps
        curl \
        ca-certificates \
        openssl \
        && \
    apt-get download \
        \
        pkg-config \
        libsqlite3-0 \
        libsqlite3-dev \
        && \
    mkdir -p /dpkg/var/lib/dpkg/status.d/ && \
    for deb in *.deb; do \
        package_name=$(dpkg-deb -I ${deb} | awk '/^ Package: .*$/ {print $2}'); \
        echo "Process: ${package_name}"; \
        dpkg --ctrl-tarfile $deb | tar -Oxf - ./control > /dpkg/var/lib/dpkg/status.d/${package_name}; \
        dpkg --extract $deb /dpkg || exit 10; \
    done

# remove not needed files extracted from deb packages like man pages and docs etc.
RUN find /dpkg/ -type d -empty -delete && \
    rm -r /dpkg/usr/share/doc/

FROM gcr.io/distroless/cc-debian10
COPY --from=runtime-deps ["/dpkg/", "/"]
COPY --from=builder /app/target/release/seqtf_url /app/seqtf_url
WORKDIR /app
CMD ["./seqtf_url"]