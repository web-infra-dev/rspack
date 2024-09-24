import "./a"
import { isAsyncModule, hasAsyncModuleRuntime } from "../utils"

module; // bailout inline entry modules optimization
const fs = __non_webpack_require__("fs");

it("should have correct value", async () => {
  expect(globalThis.case2).toBe(parseInt(WATCH_STEP, 10))
  const content = await fs.promises.readFile(__filename, "utf-8")
  switch (WATCH_STEP) {
    case "0":
    case "1":
    case "2": {
      expect(isAsyncModule(content, "./case2/index.js"))
      expect(isAsyncModule(content, "./case2/a.js"))
      expect(isAsyncModule(content, "./case2/b.js"))
      expect(isAsyncModule(content, "./case2/c.js"))
      expect(hasAsyncModuleRuntime(content))
      break;
    }
  }
})
