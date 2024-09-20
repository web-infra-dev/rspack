import(/* webpackChunkName: "chunk-a" */'./a');

it("should inject mock runtime module", async function () {
  expect(typeof __webpack_require__.mock).toBe("function");
  expect(__webpack_require__.mock("chunk-a")).toBe("chunk-a.bundle0.js");
});