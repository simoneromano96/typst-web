# Stage 1: Builder
FROM rust:1 AS build
WORKDIR /app
COPY . .
RUN cargo build --release

# Stage 2: Download typst
FROM rust:1 AS typst
RUN cargo install --locked typst-cli
RUN cp /usr/local/cargo/bin/typst /usr/local/bin/typst

# Stage 3: Final
FROM gcr.io/distroless/cc-debian12 AS final
COPY --from=build /app/target/release/typst-web /
COPY --from=typst /usr/local/bin/typst /usr/local/bin/typst
EXPOSE 3030
CMD ["./typst-web"]
