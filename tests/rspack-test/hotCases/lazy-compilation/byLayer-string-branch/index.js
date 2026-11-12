it("should reserve a string-valued layer branch", async () => {
  const promise = import("./module");
  await NEXT_HMR_DEFERRED();
  expect((await promise).value).toBe("a/b");
});
