it("should not require an inlined const module from a concatenated module", async () => {
  const [{ read }, { readOther }] = await Promise.all([
    import("pkg/async-root"),
    import("pkg/async-other")
  ]);

  expect(read()).toBe(42);
  expect(readOther()).toBe(42);
});
