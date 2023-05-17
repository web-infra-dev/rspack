#!/usr/bin/env bash

rm -rf ../../node_modules/@angular-devkit

cd ~/work/angular/angular-cli/
yarn build

cd ~/work/rspack/rspack/
pnpm install
