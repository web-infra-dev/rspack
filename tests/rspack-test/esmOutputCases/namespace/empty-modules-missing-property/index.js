import * as js from "./empty.js";
import * as mjs from "./empty.mjs";
import * as cjs from "./empty.cjs";

it("should render missing namespace properties as undefined for empty modules", () => {
  expect(js.nope).toBeUndefined();
  expect(mjs.nope).toBeUndefined();
  expect(cjs.nope).toBeUndefined();
});
