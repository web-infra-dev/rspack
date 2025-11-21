it("should handle indirect children with multiple parents correctly", async () => {
  const b = await import('./pageB');
  expect(b.default).toBe("reuse");
})
