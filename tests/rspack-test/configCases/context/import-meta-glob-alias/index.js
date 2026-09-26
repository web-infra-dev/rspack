const modules = import.meta.glob(
  ["@/glob-imports/*.js", "!@/glob-imports/excluded.js"],
  { eager: true },
);

it("resolves an alias in import.meta.glob and honors exclusions", () => {
  expect(Object.keys(modules)).toEqual(["/src/glob-imports/value.js"]);
  expect(modules["/src/glob-imports/value.js"].default).toBe("resolved");
});
