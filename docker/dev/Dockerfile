ARG RUST_BUILDER=ekidd/rust-musl-builder:latest

FROM $RUST_BUILDER as planner

# Cache deps
WORKDIR /app

#install cargo chef
RUN cargo install cargo-chef

RUN sudo chown -R rust:rust .

WORKDIR /app/server

COPY server/ .

# list off needed dependencies
RUN cargo chef prepare --recipe-path recipe.json


FROM ${RUST_BUILDER} as cacher
WORKDIR /app/server

RUN cargo install cargo-chef

COPY --from=planner /app/server/recipe.json recipe.json
RUN sudo chown -R rust:rust .
RUN cargo chef cook --recipe-path recipe.json


FROM ${RUST_BUILDER} as builder

WORKDIR /app/server

COPY server/ .

COPY --from=cacher /app/server/target ./target
COPY --from=cacher /home/rust/.cargo /home/rust/.cargo

RUN sudo chown -R rust:rust .
RUN cargo build


FROM ${RUST_BUILDER} as docs
WORKDIR /app
COPY docs ./docs
RUN sudo chown -R rust:rust .
RUN mdbook build docs/


FROM alpine:3.12 as runner

# Install libpq for postgres
RUN apk add libpq

# Install Espeak for captchas
RUN apk add espeak

# Copy resources
COPY server/config/defaults.hjson /config/defaults.hjson
COPY --from=builder /app/server/target/x86_64-unknown-linux-musl/debug/lemmy_server /app/lemmy
COPY --from=docs /app/docs/book/ /app/dist/documentation/

RUN addgroup -g 1000 lemmy
RUN adduser -D -s /bin/sh -u 1000 -G lemmy lemmy
RUN chown lemmy:lemmy /app/lemmy
USER lemmy
EXPOSE 8536
CMD ["/app/lemmy"]
