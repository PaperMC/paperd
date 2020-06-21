ARG version
FROM ubuntu:$version

# Base utilities
ENV DEBIAN_FRONTEND="noninteractive"
RUN apt-get update \
    && apt-get install --no-install-recommends -y \
        apt-transport-https \
        ca-certificates \
        curl \
        build-essential \
        gnupg \
        libncurses-dev \
        libncursesw5-dev \
        software-properties-common \
        wget

# Install Rust
ARG rustVersion
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    RUST_VERSION=$rustVersion
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --no-modify-path \
    && chmod -R a+w $RUSTUP_HOME $CARGO_HOME

# Install AdoptOpenJDK 8
RUN wget -qO - https://adoptopenjdk.jfrog.io/adoptopenjdk/api/gpg/key/public | apt-key add - \
    && add-apt-repository --yes https://adoptopenjdk.jfrog.io/adoptopenjdk/deb/ \
    && apt-get update \
    && apt-get install --no-install-recommends -y adoptopenjdk-8-hotspot

WORKDIR /usr/src/paperd
CMD ["./release/targets/build_release.sh"]
