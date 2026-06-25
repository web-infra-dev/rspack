import { a, b } from "./decl";

function foo() {
  return 1;
}

a([b("PURE_NESTING_LOCAL_B_MARKER", foo())], "PURE_NESTING_LOCAL_A_MARKER");
