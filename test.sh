#!/bin/bash

# This script performs some simple tests against the provided SpaceAPI.
# After building and running the app, it checks the given endpoints
# and validates some of its returned content.
#
# Before running the script, make sure to fill out the configuration file
# so that the app is able to connect to a database filled with some data.
# Note, that this script assumes the app to be configured to listen on localhost:3000.

# Quit on error
set -e

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
URL_STATUS="${URL}/status.json"

# Overall result
RESULT=true

# Overall error handler
trap cleanup EXIT

# Build the app
printc "$ORANGE" "=> Build"
cargo build > /dev/null 2>&1 || err "Failed to build app"

# Run the app
printc "$ORANGE" "=> Run"
cargo run > /dev/null 2>&1 &
PID=$!
sleep 0.5
ps -p "$PID" > /dev/null || err "Failed to start app"

# Perform tests
printc "$ORANGE" "=> Test"

# Test /health
curl -f -s -o /dev/null $URL_HEALTH || err "App not healthy"

# Test /status.json
if [ $RESULT = true ] ; then
    STATUS=$(curl -f -s $URL_STATUS)
    [ "$(jq '.state | has("open") and has("lastchange")' <<< "$STATUS")" = "true" ] || err "Door status missing"
    [ "$(jq '.sensors.temperature[0] | has("value") and has("unit")' <<< "$STATUS")" = "true" ] || err "Temperature value missing"
    [ "$(jq '.sensors.humidity[0] | has("value") and has("unit")' <<< "$STATUS")" = "true" ] || err "Humidity value missing"
    [ "$(jq '.sensors.carbondioxide[0] | has("value") and has("unit")' <<< "$STATUS")" = "true" ] || err "CO2 value missing"

    compat="$(jq '.api_compatibility[]' <<< "$STATUS" | tr " " "\n" | sed 's/"//g')"
    for ver in $compat
    do
        if curl -f -s "https://schema.spaceapi.io/$ver.json" > "schema_$ver.json" ; then
            jsonschema "schema_$ver.json" <<< "$STATUS" > /dev/null 2>&1 || err "Incompatible to v$ver"
        else
            err "Failed to download schema for v$ver"
        fi
        rm -f "schema_$ver.json"
    done
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