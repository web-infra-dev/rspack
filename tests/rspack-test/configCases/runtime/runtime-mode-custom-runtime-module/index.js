it("supports custom runtime modules", () => {
  expect(__webpack_require__.custom).toBe(1);
});
