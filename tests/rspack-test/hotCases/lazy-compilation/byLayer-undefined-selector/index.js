it("should omit an undefined layer selector from reserved requests", async () => {
  const promise = import("./module");
  await new Promise(resolve => setTimeout(resolve, 1000));
  await NEXT_HMR();
  expect((await promise).value).toBe("a/b");
});
