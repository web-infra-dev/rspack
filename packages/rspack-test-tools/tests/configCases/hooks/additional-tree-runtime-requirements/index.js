it("should modify runtime requirements", () => {
  // RuntimeGlobals.chunkName been added
  expect(__webpack_require__.cn).toBe("main");
  try {
    // RuntimeGlobals.getFullHash been deleted
    __webpack_hash__
  } catch (e) {
    expect(e.message).toBe("__webpack_require__.h is not a function");
  }
});
