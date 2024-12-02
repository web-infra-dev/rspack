import "./a"
import { isAsyncModule, hasAsyncModuleRuntime } from "../utils"

module; // bailout inline entry modules optimization
const fs = __non_webpack_require__("fs");

it("should have correct value", async () => {
  expect(globalThis.case2).toBe(parseInt(WATCH_STEP, 10))
  const content = await fs.promises.readFile(__filename, "utf-8")
  const result = true;
  expect(isAsyncModule(content, "./case2/index.js")).toBe(result)
  expect(isAsyncModule(content, "./case2/a.js")).toBe(result)
  expect(isAsyncModule(content, "./case2/b.js")).toBe(result)
  expect(isAsyncModule(content, "./case2/c.js")).toBe(result)
  expect(hasAsyncModuleRuntime(content)).toBe(result)
})
