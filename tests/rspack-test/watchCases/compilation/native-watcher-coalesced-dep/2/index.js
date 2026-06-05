// Only seen as 2 if `late.js` stayed watched after the coalesced build (#12904).
import { v } from "./late.js";
it("step2 detects the late.js change", () => {
  expect(v).toBe(2);
});
