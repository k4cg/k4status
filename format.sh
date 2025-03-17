#!/bin/bash

result=0

json=("template.json" "config.json")
for file in "${json[@]}" ; do
    if formatted=$(jq --indent 4 < "$file") ; then
        echo "$formatted" > "$file"
    else
        echo "Failed to format $file"
        result=1
    fi
done

svg=("badges/open.svg" "badges/closed.svg" "badges/unknown.svg")
for file in "${svg[@]}" ; do
    if formatted=$(XMLLINT_INDENT="    " xmllint --format "$file") ; then
        echo "$formatted" > "$file"
    else
        echo "Failed to format $file"
        result=1
    fi
done

exit $result
