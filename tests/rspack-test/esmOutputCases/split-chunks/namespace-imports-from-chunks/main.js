it("should support namespace imports from sibling entry chunks", async () => {
  const other = await import(/* webpackIgnore: true */ "./other.mjs");

  expect(other.default).toBe(42);
});
