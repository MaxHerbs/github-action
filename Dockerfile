FROM ubuntu:latest as build

RUN apt update && apt install curl gzip -y

WORKDIR /app

ENV PATH="/app:$PATH"
ENV KUBECONFIG="/app/kubeconfig.yaml"

COPY kubeconfig.yaml .

COPY install-argo.sh .
RUN bash ./install-argo.sh

COPY install-helm.sh .
RUN bash ./install-helm.sh


FROM alpine:3.21.3 as runtime
WORKDIR /

ENV PATH="/:$PATH"

COPY --from=build /app/argo /bin/
COPY --from=build /app/helm /bin/

COPY entrypoint.sh .

ENTRYPOINT ["/app/entrypoint.sh"]
