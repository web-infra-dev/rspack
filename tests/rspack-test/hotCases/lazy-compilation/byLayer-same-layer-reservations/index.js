it("should defer local modules excluded by an earlier external matcher", async () => {
  let resolved;
  const promise = import("./module").then(result => (resolved = result));
  expect(resolved).toBe(undefined);
  await NEXT_HMR_DEFERRED();
  expect((await promise).value).toBe(7);
});
