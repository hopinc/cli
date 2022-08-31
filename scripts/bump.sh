#!/usr/bin/env bash

if [ -z "$1" ]; then
    echo "Usage: $0 <tag>"
    exit 1
fi

TAG=$1

echo "Bumping to $TAG"

sed -i "s/^version = .*/version = \"$TAG\"/" Cargo.toml

sleep 10

git add Cargo.*
git commit -m "feat: v$TAG" -S
git tag -a v$TAG -m v$TAG -s

echo "Done!"
echo "To push the changes and tag, run: git push --follow-tags"