import { memberPure } from "./member-decl";

memberPure([
  false && globalThis.__PURE_FUNCTION_EDGE_CALLS__.push("DEAD_LOGICAL_AND_MARKER"),
  true || globalThis.__PURE_FUNCTION_EDGE_CALLS__.push("DEAD_LOGICAL_OR_MARKER"),
  true ?? globalThis.__PURE_FUNCTION_EDGE_CALLS__.push("DEAD_NULLISH_MARKER"),
]);
