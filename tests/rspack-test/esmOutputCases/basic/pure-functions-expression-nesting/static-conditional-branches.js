import { memberPure } from "./member-decl";

memberPure([
  true ? 1 : globalThis.__PURE_FUNCTION_EDGE_CALLS__.push("DEAD_COND_ALT_MARKER"),
  false ? globalThis.__PURE_FUNCTION_EDGE_CALLS__.push("DEAD_COND_CONS_MARKER") : 1,
]);
