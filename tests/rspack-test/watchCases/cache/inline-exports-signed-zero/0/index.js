import { zero } from "./value";

it("should update an inlined zero when only its sign changes", () => {
  expect(Object.is(zero, -0)).toBe(WATCH_STEP === "1");
  const source = require("fs").readFileSync(__filename, "utf-8");
  const literal = WATCH_STEP === "1" ? "-0" : "0";
  expect(source).toContain(`(/* inlined export .zero */${literal})`);
});
