ARG BASE_IMAGE_TAG=alpine

FROM golang:${BASE_IMAGE_TAG}

VOLUME /config

RUN apk update \
    && apk add git \
    && rm -rf /var/cache/apt/*

ARG GODNS_VERSION=master

RUN set -xeu; \
    git clone \
        --depth=1 \
        --branch ${GODNS_VERSION} \
        --single-branch \
        --recurse-submodules \
        --shallow-submodules \
        https://github.com/TimothyYe/godns.git \
        /godns; \
    cd /godns; \
    echo "Cloned godns $(git describe --tags)"

RUN cd /godns/cmd/godns \
    && go version \
    && go install -v

ENTRYPOINT ["godns"]
CMD ["-c", "/config/config.yaml"]
