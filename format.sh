#!/bin/bash

result=true

json=("template.json" "config.json")
for file in "${json[@]}" ; do
    content=$(cat "$file")
    if ! jq --indent 4 <<< "$content" > "$file" ; then
        echo "Failed to format $file"
        result=false
    fi
done

svg=("badges/open.svg" "badges/closed.svg" "badges/unknown.svg")
for file in "${svg[@]}" ; do
    content=$(cat "$file")
    if ! XMLLINT_INDENT="    " xmllint --format - <<< "$content" > "$file" ; then
        echo "Failed to format $file"
        result=false
    fi
done

if [ $result = true ] ; then
    exit 0
else
    exit 1
fi