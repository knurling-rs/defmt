#!/bin/bash
cd book
curl -L https://github.com/rust-lang/mdBook/releases/download/v0.5.1/mdbook-v0.5.1-x86_64-unknown-linux-gnu.tar.gz | tar xvz
echo "818d38d93524154bcb4847444bc0645b14da9ad729dacc7155ce477c26ac9470  mdbook" | sha256sum -c -
echo "commit $(git rev-parse --short HEAD) on $(git show -s --format="%ci" HEAD | cut -d" " -f1-2)" >> version.md
./mdbook build
