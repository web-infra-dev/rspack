import { unsafePureFn } from "./unsafe-decl";

const coercion = {
  valueOf() {
    (globalThis.__PURE_FUNCTION_EDGE_CALLS__ ||= []).push("UNSAFE_NUMERIC_UNARY_MARKER");
    return 1;
  },
  toString() {
    (globalThis.__PURE_FUNCTION_EDGE_CALLS__ ||= []).push("UNSAFE_BINARY_MARKER");
    return "key";
  },
};

const key = {
  toString() {
    (globalThis.__PURE_FUNCTION_EDGE_CALLS__ ||= []).push("UNSAFE_COMPUTED_KEY_MARKER");
    return "key";
  },
};

const unusedNumericUnary = +coercion;
const unusedUnaryPureCall = !unsafePureFn("UNSAFE_UNARY_PURE_CALL_MARKER");
const unusedUnsafeBinary = unsafePureFn("unsafe-binary") + coercion;
const unusedComputedKey = { [key]: unsafePureFn("unsafe-computed") };
