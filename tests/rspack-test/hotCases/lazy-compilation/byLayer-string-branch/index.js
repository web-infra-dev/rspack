it("should reserve a string-valued layer branch", async () => {
  const promise = import("./module");
  await new Promise(resolve => setTimeout(resolve, 1000));
  await NEXT_HMR();
  expect((await promise).value).toBe("a/b");
});
