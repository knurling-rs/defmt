#!/bin/bash

# Checks that all the github links in CHANGELOG.md are correctly defined

status=0

for link in `rg '\[(#\d+)\] ' CHANGELOG.md -o | sort | uniq | sed 's/\]/\\\\]/' | sed 's/\[/\\\\[/'`
do
    if ! rg "${link}:" CHANGELOG.md -q; then
        echo ${link//\\/} is MISSING a link-reference, see https://github.github.com/gfm/#shortcut-reference-link
        status=1
    fi
done

exit $status
