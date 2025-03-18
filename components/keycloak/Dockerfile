FROM quay.io/phasetwo/phasetwo-keycloak:26.0.6 AS builder

ENV KC_DB=postgres
ENV KC_HTTP_ENABLED=true
ENV KC_HEALTH_ENABLED=true
ENV KC_METRICS_ENABLED=true
ENV KC_CACHE=local
ENV KC_CACHE_STACK=any

RUN /opt/keycloak/bin/kc.sh build \
    --spi-email-template-provider=freemarker-plus-mustache \
    --spi-email-template-freemarker-plus-mustache-enabled=true


FROM quay.io/phasetwo/phasetwo-keycloak:26.0.6
COPY --from=builder /opt/keycloak/ /opt/keycloak/

# Copy the built theme
# The theme source is located in a seperate repository
COPY ./components/keycloak/keycloak-theme.jar /opt/keycloak/providers/keycloak-theme.jar

CMD ["start", "--optimized"]