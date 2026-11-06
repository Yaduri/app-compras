# ==============================================================================
# Dockerfile Multi-Stage de Produção para Railway.app
# ==============================================================================

# 1. Estágio de Compilação
FROM rust:1.82-slim AS builder

WORKDIR /app

# Dependências de sistema para compilação C / OpenSSL
RUN apt-get update && apt-get install -y pkg-config libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

# Copia arquivos do workspace necessários para o backend
COPY Cargo.toml Cargo.lock ./
COPY shared ./shared
COPY backend ./backend

# Remove o frontend do workspace do container para compilar exclusivamente o backend
RUN sed -i '/"frontend"/d' Cargo.toml

# Compilação otimizada em release
RUN cargo build --release -p backend

# 2. Estágio Final de Execução (Slim e Seguro)
FROM debian:bookworm-slim

WORKDIR /app

# Certificados de autoridade raiz para conexões TLS/HTTPS com Supabase
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

# Copia o binário do estágio de build
COPY --from=builder /app/target/release/backend /app/backend

# O Railway injeta a porta dinamicamente na variável PORT
ENV PORT=3000
EXPOSE 3000

CMD ["/app/backend"]
