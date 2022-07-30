#!/usr/bin/env bash

if [ -z "$1" ]; then
    echo "Usage: $0 <tag>"
    exit 1
fi

TAG=$1

echo "Bumping to $TAG"

if ! command -v cargo bump -h 2> /dev/null; then
    echo "error: you do not have 'cargo bump' installed which is required for this script."
    exit 1
fi

cargo bump $TAG

git add Cargo.*
git commit -m "feat: v$TAG" -S
git tag -a v$TAG -m v$TAG -s

echo "Done!"
echo "To push the changes and tag, run: git push --follow-tags"