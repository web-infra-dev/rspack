// `late.js` is first imported here, in the build the loader coalesces.
import { v } from "./late.js";
it("step1 imports late.js", () => {
  expect(v).toBe(1);
});
