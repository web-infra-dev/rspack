it("hot updates runtime code that starts and stops using webpack helpers", async () => {
  expect(require("./value")).toBe(1);
  expect(globalThis.bridgeRevision).toBe(1);
  expect(__webpack_public_path__).toBe("https://test.cases/path/");
  await NEXT_HMR();
  expect(require("./value")).toBe(2);
  expect(globalThis.bridgeRevision).toBe(2);
  expect(__webpack_public_path__).toBe("https://example.com/");
  const [fromInterceptor, readLater] = require("./subject.cjs");
  expect(fromInterceptor).toBe(41);
  expect(readLater()).toBe(42);

  await NEXT_HMR();
  expect(require("./value")).toBe(3);
  expect(globalThis.bridgeRevision).toBe(3);
  expect(readLater()).toBe(42);
  expect(module.hot.status()).toBe("idle");
});
module.hot.accept("./value");
