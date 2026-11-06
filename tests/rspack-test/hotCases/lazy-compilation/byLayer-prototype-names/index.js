it("should reserve externals for layers named after prototype properties", async () => {
  const promise = import("./module");
  await new Promise(resolve => setTimeout(resolve, 1000));
  await NEXT_HMR();
  expect((await promise).value).toBe("a/b");
});
