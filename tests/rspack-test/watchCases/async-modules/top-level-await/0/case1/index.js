import "./a"
import { isAsyncModule, hasAsyncModuleRuntime } from "../utils"

module; // bailout inline entry modules optimization
const fs = __non_webpack_require__("fs");

it("should have correct value", async () => {
  expect(globalThis.case1).toBe(parseInt(WATCH_STEP, 10))
  const content = await fs.promises.readFile(__filename, "utf-8")
  const result = WATCH_STEP === "1";
  expect(isAsyncModule(content, "./case1/index.js")).toBe(result)
  expect(isAsyncModule(content, "./case1/a.js")).toBe(result)
  expect(isAsyncModule(content, "./case1/b.js")).toBe(result)
  expect(hasAsyncModuleRuntime(content)).toBe(result)
})
