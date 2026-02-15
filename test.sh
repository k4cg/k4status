#!/bin/bash

# This script performs some simple tests against the provided SpaceAPI.
# After building and running the app, it checks the given endpoints
# and validates some of its returned content.
#
# Before running the script, make sure to fill out the configuration file
# so that the app is able to connect to a database filled with some data.
# Note, that this script assumes the app to be configured to listen on localhost:3000.

# Quit on error
set -eu

# Check for required tools
command -v curl > /dev/null
command -v xmllint > /dev/null
command -v jq > /dev/null
command -v jsonschema > /dev/null

# Cleanup, kill app
function cleanup()
{
    if [ -n "$PID" ] ; then
        kill -SIGTERM "$PID" > /dev/null 2>&1 || err "Tried to kill non-running app"
        unset PID
    fi
}

# Print a colored string
function printc
{
    echo -e "$1$2$NC"
}

# Print error and set overall result to false
function err()
{
    printc "$RED" "FAIL: $1"
    RESULT=false
}

# Color stuff
RED='\033[0;31m' # Red
ORANGE='\033[0;33m' # Orange
GREEN='\033[0;32m' # Green
NC='\033[0m' # No Color

# URLs
URL="http://localhost:3000"
URL_HEALTH="${URL}/health"
URL_STATUS="${URL}/status"
URL_BADGE="${URL}/badge"
URL_ICON_OPEN="${URL}/icon/open"
URL_ICON_CLOSED="${URL}/icon/closed"

# Overall result
RESULT=true

# Overall error handler
trap cleanup EXIT

# Build internal tests
printc "$ORANGE" "=> Build internal tests"
cargo test --no-run > /dev/null 2>&1 || err "Failed to build internal tests"

# Run internal tests
printc "$ORANGE" "=> Run internal tests"
cargo test > /dev/null 2>&1 || err "Internal tests failed"

# Build the app
printc "$ORANGE" "=> Build app"
cargo build > /dev/null 2>&1 || err "Failed to build app"

# Run the app
printc "$ORANGE" "=> Run app"
cargo run > /dev/null 2>&1 &
PID=$!
sleep 0.5
ps -p "$PID" > /dev/null || err "Failed to start app"

# Perform tests
printc "$ORANGE" "=> Test app"

# Test /health
curl -f -s -o /dev/null $URL_HEALTH || err "App not healthy"

# Test /status
if [ $RESULT = true ] ; then
    STATUS=$(curl -f -s $URL_STATUS)
    [ "$(jq '.state | has("open") and has("lastchange")' <<< "$STATUS")" = "true" ] || err "Door status missing"
    [ "$(jq '.sensors.temperature[0] | has("value") and has("unit")' <<< "$STATUS")" = "true" ] || err "Temperature value missing"
    [ "$(jq '.sensors.humidity[0] | has("value") and has("unit")' <<< "$STATUS")" = "true" ] || err "Humidity value missing"
    [ "$(jq '.sensors.carbondioxide[0] | has("value") and has("unit")' <<< "$STATUS")" = "true" ] || err "CO2 value missing"

    if [ "$(jq '.api_compatibility | length' <<< "$STATUS")" -gt 0 ] ; then
        compat="$(jq '.api_compatibility[]' <<< "$STATUS" | tr " " "\n" | sed 's/"//g')"
        for ver in $compat
        do
            if curl -f -s "https://raw.githubusercontent.com/SpaceApi/schema/refs/heads/master/$ver.json" > "schema_$ver.json" ; then
                jsonschema "schema_$ver.json" <<< "$STATUS" > /dev/null 2>&1 || err "Incompatible to v$ver"
            else
                err "Failed to download schema for v$ver"
            fi
            rm -f "schema_$ver.json"
        done
    else
        err "API compatiblity empty"
    fi
fi

# Test /badge
if [ $RESULT = true ] ; then
    door_state="$(curl -f -s $URL_STATUS | jq '.state.open')"
    door_badge="$(curl -f -s $URL_BADGE | sed 's/xmlns=".*"//g' | xmllint --xpath "/svg/title/text()" -)"

    if [ "$door_state" = "true" ] ; then
        grep -q "open" <<< "$door_badge" || err "Door/badge state mismatch"
    else
        grep -q "closed" <<< "$door_badge" || err "Door/badge state mismatch"
    fi
fi

# Test /icon
if [ $RESULT = true ] ; then
    curl -s -f $URL_ICON_OPEN -o /dev/null || err "Icon 'open' missing"
    curl -s -f $URL_ICON_CLOSED -o /dev/null || err "Icon 'closed' missing"
fi

# Cleanup
trap - EXIT
cleanup
if [ $RESULT = true ] ; then
    printc "$GREEN" "=> Result: PASS"
    exit 0
else
    printc "$RED" "=> Result: FAIL"
    exit 1
fi