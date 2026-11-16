it("should reserve externals for synchronous descendants in different layers", async () => {
  let resolved;
  const promise = import("./module").then(result => (resolved = result));
  expect(resolved).toBe(undefined);
  await NEXT_HMR_DEFERRED();
  const result = await promise;
  expect(result.fromServer).toBe(process.platform === "win32" ? "a\\b" : "a/b");
  expect(result.fromClient).toBe("function");
  expect(result.localValue).toBe(7);
});
