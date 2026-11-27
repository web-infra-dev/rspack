import { value as wrappedAlias } from "./lib";
export const beforeWrap = wrappedAlias + 1;
const wrappedValue = wrappedAlias + 2;
globalThis.wrappedResult = wrappedValue;
debugger;
