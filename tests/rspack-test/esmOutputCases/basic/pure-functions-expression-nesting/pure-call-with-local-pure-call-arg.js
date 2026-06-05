import { a, b } from "./decl";

function foo() {
  return 1;
}

a([b(0, foo())]);
