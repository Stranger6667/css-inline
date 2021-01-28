#!/bin/bash
set -ex

# Create a symlink for css-inline
ln -sf ../../css-inline css-inline-lib
# Modify Cargo.toml to include this symlink
cp Cargo.toml Cargo.toml.orig
sed -i 's/\.\.\/\.\.\/css-inline/\.\/css-inline-lib/' Cargo.toml
# Build the source distribution
python setup.py sdist
rm css-inline-lib
mv Cargo.toml.orig Cargo.toml
