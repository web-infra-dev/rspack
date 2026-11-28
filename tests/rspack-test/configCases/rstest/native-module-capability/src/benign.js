import { isBuiltin } from "node:module";
export function isNativeBuiltin() {
  return isBuiltin("fs");
}
