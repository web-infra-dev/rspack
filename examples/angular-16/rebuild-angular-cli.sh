#!/usr/bin/env bash

rm -rf ./node_modules/@angular-devkit

cd ../../../angular-cli/
yarn build

cd ~/work/rspack/
pnpm install
