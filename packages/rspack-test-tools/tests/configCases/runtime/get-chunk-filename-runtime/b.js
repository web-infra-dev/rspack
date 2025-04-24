it("should load common chunk in b", async () => {
  const module = await import("./common");
  expect(module.common).toBe("common");
});

