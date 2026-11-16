it("should omit an undefined layer selector from reserved requests", async () => {
  const promise = import("./module");
  await NEXT_HMR_DEFERRED();
  expect((await promise).value).toBe("a/b");
});
