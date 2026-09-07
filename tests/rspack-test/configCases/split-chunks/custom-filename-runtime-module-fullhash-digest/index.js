it("should load a split chunk whose filename contains an encoded full hash", async () => {
  const { value } = await import("./async");
  expect(value).toBe(42);
});
