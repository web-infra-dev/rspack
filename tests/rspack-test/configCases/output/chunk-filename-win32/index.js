it("should generate async chunk", async function () {
  const a = await import("./a");
  expect(a.default).toBe("a");

  const b = await import("./b");
  expect(b.default).toBe("b");
});