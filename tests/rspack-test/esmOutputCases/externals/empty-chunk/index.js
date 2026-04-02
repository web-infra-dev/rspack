it("should keep an empty async chunk as ESM", async () => {
  const mod = await import("./empty");

  expect(typeof mod).toBe("object");
  expect(Object.keys(mod).filter((key) => key !== "__esModule")).toHaveLength(0);
});
