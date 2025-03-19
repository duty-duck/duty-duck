FROM quay.io/phasetwo/phasetwo-keycloak:26.0.6

ENV KC_DB=postgres
ENV KC_HTTP_ENABLED=true
ENV KC_HEALTH_ENABLED=true
ENV KC_METRICS_ENABLED=true
ENV KC_CACHE=local
ENV KC_CACHE_STACK=any

# Copy the built theme
# The theme source is located in a seperate repository
COPY ./components/keycloak/keycloak-theme.jar /opt/keycloak/providers/keycloak-theme.jar

RUN /opt/keycloak/bin/kc.sh build \
    --spi-email-template-provider=freemarker-plus-mustache \
    --spi-email-template-freemarker-plus-mustache-enabled=true

CMD ["start", "--optimized"]