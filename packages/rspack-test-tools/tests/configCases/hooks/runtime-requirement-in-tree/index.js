it("should modify runtime requirements", () => {
  // RuntimeGlobals.chunkName been added
  expect(__webpack_require__.cn).toBe("main");
  // RuntimeGlobals.ensureChunk been added
  expect(__webpack_require__.f).toBeTruthy();
  // RuntimeGlobals.hasOwnProperty been added as dependency
  expect(__webpack_require__.o).toBeTruthy();
  expect(__webpack_require__.custom).toBe(42);
});
