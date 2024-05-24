# List the available commands
help:
    just --list

# Start the required Docker services
dockerUp:
    docker-compose up -d

# Run the end to end tests
endToEndTests: dockerUp
    cargo test --test end_to_end -- --concurrency=1

# Run the unit tests
unitTests:
    cargo test --bin uptime-monitoring

# Run all the tests
test: endToEndTests unitTests