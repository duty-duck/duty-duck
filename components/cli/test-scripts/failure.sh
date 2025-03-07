#! /bin/bash
# Failing script designed to test the task features 
# You can launch it with:
# cargo run --bin dutyduck tasks run --create --task-id test-failed-script ./components/cli/test-scripts/failure.sh

echo "Hello, world!"

sleep 10;

echo "See you in hell, world!"

exit 1;