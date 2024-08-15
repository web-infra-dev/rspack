var __webpack_require__ = {};
__webpack_require__.e = 42;

it("rename top level (avoid override by inner graph top level symbol)", () => {
  expect(__webpack_require__.e).toBe(42);
  const lib2 = require("./lib2");
  expect(lib2.a).toBe(1)
});
