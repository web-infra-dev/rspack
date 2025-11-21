import "./a"
import { isAsyncModule, hasAsyncModuleRuntime } from "../utils"

module; // bailout inline entry modules optimization
const fs = __non_webpack_require__("fs");

it("should have correct value", async () => {
  expect(globalThis.case3).toBe(parseInt(WATCH_STEP, 10))
  const content = await fs.promises.readFile(__filename, "utf-8")
  const result = WATCH_STEP === "1";
  expect(isAsyncModule(content, "./case3/index.js")).toBe(true)
  expect(isAsyncModule(content, "./case3/a.js")).toBe(true)
  expect(isAsyncModule(content, "./case3/b.js")).toBe(result)
  expect(isAsyncModule(content, "./case3/c.js")).toBe(true)
  expect(hasAsyncModuleRuntime(content)).toBe(true)
})
