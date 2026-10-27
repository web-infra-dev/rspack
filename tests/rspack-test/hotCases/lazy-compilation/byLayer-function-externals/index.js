it("should reserve base and layer externals before the first lazy activation", async () => {
  let resolved;
  const promise = import("./module").then(result => (resolved = result));
  expect(resolved).toBe(undefined);
  await new Promise(resolve => setTimeout(resolve, 1000));
  await NEXT_HMR();
  const result = await promise;
  expect(typeof result.isFile).toBe("function");
  expect(result.joinedPath).toBe(require("path").join("a", "b"));
});
