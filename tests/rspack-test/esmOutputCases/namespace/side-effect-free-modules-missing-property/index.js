import * as js from "./foo/no-side-effects.js";
import * as mjs from "./foo/no-side-effects.mjs";
import * as cjs from "./foo/no-side-effects.cjs";

it("should render missing namespace properties as undefined for side-effect-free modules", () => {
  expect(js.nope).toBeUndefined();
  expect(mjs.nope).toBeUndefined();
  expect(cjs.nope).toBeUndefined();
});
