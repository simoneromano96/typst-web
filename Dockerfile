FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

# Stage 1: Build planner
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 2: Build binary
FROM chef AS build
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --bin typst-web

# Stage 3: Download typst
FROM chef AS typst
RUN cargo install --locked typst-cli
RUN cp /usr/local/cargo/bin/typst /usr/local/bin/typst

# Stage 4: Final
FROM gcr.io/distroless/cc-debian12 AS final
COPY --from=build /app/target/release/typst-web /
COPY --from=typst /usr/local/bin/typst /usr/local/bin/typst
EXPOSE 3030
CMD ["./typst-web"]
