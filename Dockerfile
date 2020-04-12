FROM rust:1.42.0-slim-buster

# Base utilities
RUN mkdir -p /usr/share/man/man1 \
    && apt-get update \
    && apt-get install --no-install-recommends -y \
        apt-utils \
        gnupg \
        libncurses-dev \
        software-properties-common \
        wget \
        xz-utils

# Install AdoptOpenJDK 8
RUN wget -qO - https://adoptopenjdk.jfrog.io/adoptopenjdk/api/gpg/key/public | apt-key add - \
    && add-apt-repository --yes https://adoptopenjdk.jfrog.io/adoptopenjdk/deb/ \
    && apt-get update \
    && apt-get install --no-install-recommends -y adoptopenjdk-8-hotspot

WORKDIR /usr/src/paperd

CMD ["./build_release.sh", "__build"]
