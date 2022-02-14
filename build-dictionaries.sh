#!/bin/sh
set -e
pushd . > /dev/null
cd dictionaries/english
./build-dictionary.py
popd > /dev/null