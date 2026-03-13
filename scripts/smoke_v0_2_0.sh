#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SANDBOX_ROOT="${FA_SANDBOX_DIR:-$ROOT_DIR/sandbox}"
ADDR="${FA_SERVER_ADDR:-127.0.0.1:8000}"
HOST="${ADDR%:*}"
PORT="${ADDR##*:}"
USER_PROVIDED_DATA_DIR="${FA_DATA_DIR+x}"
DATA_DIR="${FA_DATA_DIR:-$SANDBOX_ROOT/fa-v0.2.0-smoke-$RANDOM$RANDOM}"
LOG_FILE="${FA_SMOKE_LOG_FILE:-$DATA_DIR/fa-server.log}"
TASK_ID="${FA_SMOKE_TASK_ID:-72c8f5d0-0f08-4e0c-a8c4-1d4dc51a25f0}"
BINARY_PATH="$ROOT_DIR/target/debug/fa-server"

SERVER_PID=""
CREATED_DATA_DIR="false"

cleanup() {
  if [[ -n "$SERVER_PID" ]] && kill -0 "$SERVER_PID" 2>/dev/null; then
    kill -INT "$SERVER_PID" 2>/dev/null || true
    wait "$SERVER_PID" 2>/dev/null || true
  fi
  if [[ "$CREATED_DATA_DIR" == "true" ]]; then
    rm -rf "$DATA_DIR"
  fi
}

trap cleanup EXIT

assert_contains() {
  local haystack="$1"
  local needle="$2"
  if [[ "$haystack" != *"$needle"* ]]; then
    echo "assertion failed: expected response to contain: $needle" >&2
    exit 1
  fi
}

count_matches() {
  local haystack="$1"
  local needle="$2"
  printf '%s' "$haystack" | grep -oF "$needle" || true
}

wait_for_healthz() {
  local attempts=30
  local sleep_seconds=1
  for ((i = 1; i <= attempts; i++)); do
    if curl -fsS "http://$HOST:$PORT/healthz" >/dev/null 2>&1; then
      return 0
    fi
    sleep "$sleep_seconds"
  done

  echo "fa-server failed to become healthy on $ADDR" >&2
  if [[ -f "$LOG_FILE" ]]; then
    echo "---- server log ----" >&2
    tail -n 200 "$LOG_FILE" >&2 || true
  fi
  exit 1
}

start_server() {
  mkdir -p "$DATA_DIR"
  if [[ "$USER_PROVIDED_DATA_DIR" != "x" ]]; then
    CREATED_DATA_DIR="true"
  fi
  FA_SERVER_ADDR="$ADDR" FA_DATA_DIR="$DATA_DIR" "$BINARY_PATH" >"$LOG_FILE" 2>&1 &
  SERVER_PID="$!"
  wait_for_healthz
}

stop_server() {
  if [[ -n "$SERVER_PID" ]] && kill -0 "$SERVER_PID" 2>/dev/null; then
    kill -INT "$SERVER_PID" 2>/dev/null || true
    wait "$SERVER_PID" 2>/dev/null || true
  fi
  SERVER_PID=""
}

if command -v lsof >/dev/null 2>&1; then
  if lsof -iTCP:"$PORT" -sTCP:LISTEN >/dev/null 2>&1; then
    echo "port $PORT is already in use; set FA_SERVER_ADDR to override for smoke testing" >&2
    exit 1
  fi
fi

(
  cd "$ROOT_DIR"
  cargo build -p fa-server >/dev/null
)

INTAKE_PAYLOAD=$(cat <<JSON
{
  "id": "$TASK_ID",
  "title": "Investigate spindle temperature drift",
  "description": "Diagnose repeated spindle temperature drift before the next shift.",
  "priority": "critical",
  "risk": "high",
  "initiator": {
    "id": "worker_1001",
    "display_name": "Liu Supervisor",
    "role": "Production Supervisor"
  },
  "stakeholders": [],
  "equipment_ids": ["eq_cnc_01"],
  "integrations": ["mes", "cmms"],
  "desired_outcome": "Recover stable spindle temperature within tolerance",
  "requires_human_approval": true,
  "requires_diagnostic_loop": true
}
JSON
)

APPROVE_PAYLOAD=$(cat <<JSON
{
  "decided_by": {
    "id": "worker_2001",
    "display_name": "Wang Safety",
    "role": "Safety Officer"
  },
  "approved": true,
  "comment": "Proceed to execution"
}
JSON
)

EXECUTE_PAYLOAD=$(cat <<JSON
{
  "actor": {
    "id": "worker_3001",
    "display_name": "Wu Maint",
    "role": "Maintenance Technician"
  },
  "note": "Execution stub started"
}
JSON
)

COMPLETE_PAYLOAD=$(cat <<JSON
{
  "actor": {
    "id": "worker_3001",
    "display_name": "Wu Maint",
    "role": "Maintenance Technician"
  },
  "note": "Execution finished"
}
JSON
)

start_server

intake_response="$(curl -fsS "http://$HOST:$PORT/api/v1/tasks/intake" \
  -H "Content-Type: application/json" \
  -H "x-correlation-id: smoke-intake-001" \
  -d "$INTAKE_PAYLOAD")"
assert_contains "$intake_response" '"status":"awaiting_approval"'
assert_contains "$intake_response" '"correlation_id":"smoke-intake-001"'
if [[ "$(count_matches "$intake_response" '"source_ref"' | wc -l | tr -d ' ')" -lt 4 ]]; then
  echo "assertion failed: expected at least 4 evidence items in intake response" >&2
  exit 1
fi

task_response="$(curl -fsS "http://$HOST:$PORT/api/v1/tasks/$TASK_ID")"
assert_contains "$task_response" "\"id\":\"$TASK_ID\""
if [[ "$(count_matches "$task_response" '"source_ref"' | wc -l | tr -d ' ')" -lt 4 ]]; then
  echo "assertion failed: expected at least 4 evidence items in task response" >&2
  exit 1
fi

evidence_response="$(curl -fsS "http://$HOST:$PORT/api/v1/tasks/$TASK_ID/evidence")"
if [[ "$(count_matches "$evidence_response" '"source_ref"' | wc -l | tr -d ' ')" -lt 4 ]]; then
  echo "assertion failed: expected at least 4 evidence items from evidence endpoint" >&2
  exit 1
fi
assert_contains "$evidence_response" 'telemetry'

governance_response="$(curl -fsS "http://$HOST:$PORT/api/v1/tasks/$TASK_ID/governance")"
assert_contains "$governance_response" '"required_role":"safety_officer"'
assert_contains "$governance_response" '"maintenance_engineer"'

stop_server
start_server

persisted_task_response="$(curl -fsS "http://$HOST:$PORT/api/v1/tasks/$TASK_ID")"
assert_contains "$persisted_task_response" "\"id\":\"$TASK_ID\""
assert_contains "$persisted_task_response" '"status":"awaiting_approval"'

persisted_evidence_response="$(curl -fsS "http://$HOST:$PORT/api/v1/tasks/$TASK_ID/evidence")"
if [[ "$(count_matches "$persisted_evidence_response" '"source_ref"' | wc -l | tr -d ' ')" -lt 4 ]]; then
  echo "assertion failed: expected evidence items after restart" >&2
  exit 1
fi

persisted_governance_response="$(curl -fsS "http://$HOST:$PORT/api/v1/tasks/$TASK_ID/governance")"
assert_contains "$persisted_governance_response" '"required_role":"safety_officer"'

approve_response="$(curl -fsS "http://$HOST:$PORT/api/v1/tasks/$TASK_ID/approve" \
  -H "Content-Type: application/json" \
  -H "x-correlation-id: smoke-approve-001" \
  -d "$APPROVE_PAYLOAD")"
assert_contains "$approve_response" '"status":"approved"'

execute_response="$(curl -fsS "http://$HOST:$PORT/api/v1/tasks/$TASK_ID/execute" \
  -H "Content-Type: application/json" \
  -H "x-correlation-id: smoke-execute-001" \
  -d "$EXECUTE_PAYLOAD")"
assert_contains "$execute_response" '"status":"executing"'

complete_response="$(curl -fsS "http://$HOST:$PORT/api/v1/tasks/$TASK_ID/complete" \
  -H "Content-Type: application/json" \
  -H "x-correlation-id: smoke-complete-001" \
  -d "$COMPLETE_PAYLOAD")"
assert_contains "$complete_response" '"status":"completed"'

audit_response="$(curl -fsS "http://$HOST:$PORT/api/v1/tasks/$TASK_ID/audit-events")"
if [[ "$(count_matches "$audit_response" '"kind"' | wc -l | tr -d ' ')" -lt 8 ]]; then
  echo "assertion failed: expected at least 8 task audit events" >&2
  exit 1
fi

approval_query_response="$(curl -fsS "http://$HOST:$PORT/api/v1/audit/events?correlation_id=smoke-approve-001")"
if [[ "$(count_matches "$approval_query_response" '"kind"' | wc -l | tr -d ' ')" -lt 2 ]]; then
  echo "assertion failed: expected approval correlation query to return at least 2 events" >&2
  exit 1
fi

echo "v0.2.0 smoke succeeded on $ADDR using data dir $DATA_DIR"
