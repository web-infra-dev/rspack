it("should dynamically import another entry", async () => {
  const ns = await import("./entry-b");
  expect(ns.default).toBe(1);
});
