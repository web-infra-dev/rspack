it("updates the original plugin source in the active runtime", async () => {
  expect(require("./value")).toBe(1);
  expect(__webpack_require__.pluginRevision).toBe(1);
  expect(__webpack_require__.pluginHash).toBe(__webpack_hash__);
  await NEXT_HMR();
  expect(require("./value")).toBe(2);
  expect(__webpack_require__.pluginRevision).toBe(2);
  expect(__webpack_require__.pluginHash).toBe(__webpack_hash__);
});
module.hot.accept("./value");
