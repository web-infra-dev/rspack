it("should load common chunk in a", async () => {
  const module = await import("./common");
  expect(module.common).toBe("common");
});

