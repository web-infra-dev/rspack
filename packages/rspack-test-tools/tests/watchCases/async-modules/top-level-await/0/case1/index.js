import "./a"
import { isAsyncModule, hasAsyncModuleRuntime } from "../utils"

module; // bailout inline entry modules optimization
const fs = __non_webpack_require__("fs");

it("should have correct value", async () => {
  expect(globalThis.case1).toBe(parseInt(WATCH_STEP, 10))
  const content = await fs.promises.readFile(__filename, "utf-8")
  switch (WATCH_STEP) {
    case "0":
    case "2": {
      expect(!isAsyncModule(content, "./case1/index.js"))
      expect(!isAsyncModule(content, "./case1/a.js"))
      expect(!isAsyncModule(content, "./case1/b.js"))
      expect(!hasAsyncModuleRuntime(content))
      break;
    }
    case "1": {
      expect(isAsyncModule(content, "./case1/index.js"))
      expect(isAsyncModule(content, "./case1/a.js"))
      expect(isAsyncModule(content, "./case1/b.js"))
      expect(hasAsyncModuleRuntime(content))
      break;
    }
  }
})
