#!/bin/bash
# Run integration tests against the running AIHarness instance

PORT=${1:-8787}

echo "Running tests against http://127.0.0.1:$PORT..."

# Check if server is up
if ! curl -s "http://127.0.0.1:$PORT/" > /dev/null; then
    echo "Error: AIHarness server not found at port $PORT."
    echo "Please start the app first."
    exit 1
fi

# Run python test script
python3 scripts/test_client.py --port "$PORT"
