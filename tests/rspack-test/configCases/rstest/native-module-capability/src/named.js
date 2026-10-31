import { createRequire as makeRequire } from "node:module";
const escapedFactory = makeRequire;
const nativeRequire = escapedFactory(import.meta.url);
export function price() {
  return nativeRequire("./provider.cjs").price;
}
