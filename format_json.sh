#!/bin/bash

template=$(cat template.json)
if ! jq --indent 4 <<< "$template" > template.json ; then
    echo "Failed to format template"
    exit 1
fi

config=$(cat config.json)
if ! jq --indent 4 <<< "$config" > config.json ; then
    echo "Failed to format config"
    exit 1
fi

exit 0