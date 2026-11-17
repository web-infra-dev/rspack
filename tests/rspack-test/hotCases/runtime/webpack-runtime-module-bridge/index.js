it("updates the original plugin source in the active runtime", async () => {
  expect(require("./value")).toBe(1);
  expect(__webpack_require__.pluginRevision).toBe(1);
  expect(__webpack_require__.pluginHash).toBe(__webpack_hash__);
  const loader = __webpack_require__.initialPluginLoader;
  await NEXT_HMR();
  expect(require("./value")).toBe(2);
  expect(__webpack_require__.pluginRevision).toBe(2);
  expect(__webpack_require__.pluginHash).toBe(__webpack_hash__);
  const updatedHash = loader.h;
  expect(updatedHash()).toBe(__webpack_hash__);
  loader.h = () => "from assignment";
  expect(__webpack_hash__).toBe("from assignment");
  Object.defineProperty(loader, "h", { value: () => "from descriptor", writable: true });
  expect(__webpack_hash__).toBe("from descriptor");
  loader.h = updatedHash;
});
module.hot.accept("./value");
