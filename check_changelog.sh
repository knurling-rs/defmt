#!/bin/bash

# Checks that all the github links in CHANGELOG.md are correctly defined

for link in `rg '\[(#\d+)\] ' CHANGELOG.md -o | sort | uniq | sed 's/\]/\\\\]/' | sed 's/\[/\\\\[/'`
do
    grep "${link}:" CHANGELOG.md -q || echo ${link//\\/} is MISSING a link-reference, see https://github.github.com/gfm/#shortcut-reference-link
done
