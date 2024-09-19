// -- code start --

import "./a"

// bailout inline modules optimization
module;
// fs
const fs = __non_webpack_require__("fs");

it("should have correct value", async () => {
  expect(globalThis.b).toBe(1)
  const content = await fs.promises.readFile(__filename, "utf-8")
  switch (WATCH_STEP) {
    case "0":
    case "2": {
      expect(!isAsyncModule(content, "./index.js"))
      expect(!isAsyncModule(content, "./a.js"))
      expect(!isAsyncModule(content, "./b.js"))
      expect(!hasAsyncModuleRuntime(content))
      break;
    }
    case "1": {
      expect(isAsyncModule(content, "./index.js"))
      expect(isAsyncModule(content, "./a.js"))
      expect(isAsyncModule(content, "./b.js"))
      expect(hasAsyncModuleRuntime(content))
      break;
    }
  }
})

function escapeRegExp(string) {
  return string.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function isAsyncModule(content, moduleId) {
  const regex = new RegExp(`\\"${escapeRegExp(moduleId)}":.*\\{\\s([\\S\\s]*)\\/\\/ -- code start --`)
  const [, header] = regex.exec(content)
  return header.trim().startsWith("__webpack_require__.a(")
}

function hasAsyncModuleRuntime(content) {
  const comment = "//" + ["webpack", "runtime", "async_module"].join("/");
  return content.includes(comment)
}
