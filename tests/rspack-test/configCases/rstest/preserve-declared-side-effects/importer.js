import "./declared-true/effect.js";
import "./undeclared.js";
import "./declared-false/effect.js";
import "./rule-free.js";
import "./array/keep.js";
import "./array/omit.js";
import "registered-package";
import "./pure.js";

export function read() {
  return 42;
}
