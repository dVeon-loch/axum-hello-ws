#!/usr/bin/env bash
set -euo pipefail

BASE_URL="${1:-https://axum-hello-ws.sylvanbloch.workers.dev}"
BASE_URL="${BASE_URL%/}"

WS_URL="${BASE_URL/https:\/\//wss://}"
WS_URL="${WS_URL/http:\/\//ws://}"

PASS=0
FAIL=0

ok()   { echo "  PASS: $*"; PASS=$((PASS + 1)); }
fail() { echo "  FAIL: $*"; FAIL=$((FAIL + 1)); }

echo "Checking $BASE_URL"
echo

# REST
echo "GET /"
BODY=$(curl -sf "$BASE_URL/") && true
if [ "${BODY:-}" = "Hello, World!" ]; then
    ok "response = '$BODY'"
else
    fail "expected 'Hello, World!', got '${BODY:-<empty>}'"
fi
echo

# WebSocket
echo "WS /ws"
if ! command -v websocat &>/dev/null; then
    echo "  SKIP: websocat not found (cargo install websocat)"
else
    WS_BODY=$(echo "test" | timeout 5 websocat "$WS_URL/ws" 2>/dev/null | head -1) && true
    if [ "${WS_BODY:-}" = "hello: test" ]; then
        ok "sent 'test', got '$WS_BODY'"
    else
        fail "expected 'hello: test', got '${WS_BODY:-<empty>}'"
    fi
fi
echo

echo "$PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ]
