function expectLiveNamespace(namespace) {
  expect(Reflect.get(namespace, "__esModule")).toBe(true);
  expect(Object.prototype.toString.call(namespace)).toBe("[object Module]");

  const descriptor = Object.getOwnPropertyDescriptor(namespace, "value");
  expect(descriptor.enumerable).toBe(true);
  expect(typeof descriptor.get).toBe("function");
  expect(descriptor.set).toBeUndefined();
}

it("combines concatenated exports while preserving namespace semantics", async () => {
  const exports = await import(
    /* webpackChunkName: "concatenated" */ "./module.js"
  );
  const namespace = exports.getNamespace();

  expectLiveNamespace(exports);
  expectLiveNamespace(namespace);
  expect(exports.value).toBe(1);
  expect(namespace.value).toBe(1);

  exports.setValue(2);
  expect(exports.value).toBe(2);
  expect(namespace.value).toBe(2);
});
