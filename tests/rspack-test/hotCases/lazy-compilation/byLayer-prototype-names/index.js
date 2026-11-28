it("should reserve externals for layers named after prototype properties", async () => {
  const promise = import("./module");
  await NEXT_HMR_DEFERRED();
  expect((await promise).value).toBe("a/b");
});
