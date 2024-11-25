it("should inject runtime according to isolate scope", async function () {
  expect(__webpack_require__.mock()).toBe("isolated");
});