it("should load entry a glob", async () => {
  const modules = import.meta.glob("./modules-a/*.js");

  expect(Object.keys(modules)).toEqual(["./modules-a/a.js"]);

  const value = await modules["./modules-a/a.js"]();
  expect(value.default).toBe("a");
});
