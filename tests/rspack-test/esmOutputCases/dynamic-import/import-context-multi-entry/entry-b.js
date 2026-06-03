it("should load entry b glob", async () => {
  const modules = import.meta.glob("./modules-b/*.js");

  expect(Object.keys(modules)).toEqual(["./modules-b/b.js"]);

  const value = await modules["./modules-b/b.js"]();
  expect(value.default).toBe("b");
});
