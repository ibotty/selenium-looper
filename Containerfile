FROM registry.access.redhat.com/ubi9-micro
ARG BINARY=target/release/looper

LABEL maintainer="Tobias Florek <tob@butter.sh>"

EXPOSE 8080/tcp

COPY $BINARY /looper

CMD /looper
USER 1000
