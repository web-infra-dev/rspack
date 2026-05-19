import * as js from "./empty.js";
import * as mjs from "./empty.mjs";
import * as cjs from "./empty.cjs";

it("should keep default namespace semantics for empty modules", () => {
  expect(js.default).toBeUndefined();
  expect(mjs.default).toBeUndefined();
  expect(typeof cjs.default).toBe("object");
});
