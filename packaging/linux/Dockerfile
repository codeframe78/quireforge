FROM node:22.22.1-bookworm-slim@sha256:4f77a690f2f8946ab16fe1e791a3ac0667ae1c3575c3e4d0d4589e9ed5bfaf3d AS node_toolchain
FROM rust:1.95.0-slim-bookworm@sha256:d7482085ff5b415f84dba5647ae71606650bdef00db7aeb69f4b3d170c3e4082 AS rust_toolchain
FROM ubuntu:22.04@sha256:0e0a0fc6d18feda9db1590da249ac93e8d5abfea8f4c3c0c849ce512b5ef8982

ENV DEBIAN_FRONTEND=noninteractive
ENV PATH=/usr/local/cargo/bin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin
ENV RUSTUP_HOME=/usr/local/rustup

RUN apt-get update \
    && apt-get install --yes --no-install-recommends \
      build-essential \
      ca-certificates \
      curl \
      dbus-x11 \
      desktop-file-utils \
      dpkg-dev \
      file \
      git \
      jq \
      libayatana-appindicator3-dev \
      libfuse2 \
      librsvg2-dev \
      libssl-dev \
      libwebkit2gtk-4.1-dev \
      libxdo-dev \
      patchelf \
      python3 \
      squashfs-tools \
      wget \
      xvfb \
      xdg-utils \
      xz-utils \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Runtime contract probes are kept in a separate layer so changes to smoke
# tooling do not invalidate the much larger Tauri development-package layer.
RUN apt-get update \
    && apt-get install --yes --no-install-recommends \
      appstream \
      xdotool \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

COPY --from=node_toolchain /usr/local/ /usr/local/
COPY --from=rust_toolchain /usr/local/cargo/ /usr/local/cargo/
COPY --from=rust_toolchain /usr/local/rustup/ /usr/local/rustup/

RUN corepack enable pnpm \
    && corepack prepare pnpm@11.15.0 --activate

WORKDIR /workspace
