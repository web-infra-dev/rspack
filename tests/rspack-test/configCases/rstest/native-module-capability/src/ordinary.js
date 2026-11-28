import path from "node:path";
export function value() {
  return path.basename("/inventory/value");
}
export function shadowed(require) {
  return require("node:module");
}
