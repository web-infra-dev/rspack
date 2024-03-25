import test from "./index.module.css";
import { res } from "./lib.js";


it("should not mangle css module", () => {
  res;
  // Using this to trigger a none provided export
  test.res;

	expect(test.test).toBe("a893c4fb9453f05149a4");
});
