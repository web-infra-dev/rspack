it("loads async chunk", async () => {
  const { value } = await import("./async");

  expect(value).toBe("async");
});
