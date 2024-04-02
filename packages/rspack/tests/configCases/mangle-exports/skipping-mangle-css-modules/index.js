import test from "./index.module.css";
import { res } from "./lib.js";


it("should not mangle css module", () => {
  res;
  // Using this to trigger a none provided export
  test.res;

	expect(test.test).toBe("-_921b05f8c9c16ca9ea84-test");
});
