#!/bin/bash

# Checks that all the github links in CHANGELOG.md are correctly defined

for link in `rg '\[(#\d+)\] ' CHANGELOG.md -o | sort | uniq | sed 's/\]/\\\\]/' | sed 's/\[/\\\\[/'`
do
    rg "${link}:" CHANGELOG.md -q || echo $link is MISSING
done
