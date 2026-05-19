import * as js from "./foo/no-side-effects.js";
import * as mjs from "./foo/no-side-effects.mjs";
import * as cjs from "./foo/no-side-effects.cjs";

it("should keep default namespace semantics for side-effect-free modules", () => {
  expect(js.default).toBeUndefined();
  expect(mjs.default).toBeUndefined();
  expect(typeof cjs.default).toBe("object");
});
