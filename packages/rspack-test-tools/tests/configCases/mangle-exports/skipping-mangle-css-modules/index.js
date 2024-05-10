import test from "./index.module.css";
import { res } from "./lib.js";

it("should not mangle css module", () => {
  res;
  // Using this to trigger a none provided export
  test.res;

  expect(test.test).toBe("-ca56f23580d7bbb484-test");
});
