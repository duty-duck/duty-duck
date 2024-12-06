#! /bin/bash

cargo run --bin dutyduck_cli tasks run --create --task-id test-failed-script --name daily-sales-report ./components/cli/test-scripts/failure.sh