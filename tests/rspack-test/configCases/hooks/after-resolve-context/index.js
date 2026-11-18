it("should resolve loader-generated dependencies from createData.context", () => {
  expect(require("./stubs/instrumentation-stub.js")).toBe(42);
});

it("should apply createData.context changes in afterResolve taps", () => {
  expect(require("./stubs/hook-stub.js")).toBe(42);
});

it("should use the resource directory when createData.context is not set", () => {
  expect(require("./stubs/unchanged.js")).toBe(24);
});
