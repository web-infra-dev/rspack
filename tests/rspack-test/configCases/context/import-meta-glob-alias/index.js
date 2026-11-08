const modules = import.meta.glob(
  ["@/glob-imports/*.js", "!@/glob-imports/excluded.js"],
  { eager: true },
);

it("resolves an alias in import.meta.glob and honors exclusions", () => {
  expect(Object.keys(modules)).toEqual(["/src/glob-imports/value.js"]);
  expect(modules["/src/glob-imports/value.js"].default).toBe("resolved");
});

it("keeps bare patterns without an alias relative to the importer", async () => {
  const { nestedModules, queryModules } = await import("./src/bare/globs.js");
  expect(Object.values(nestedModules)).toEqual(["nested"]);
  const [loadQuery] = Object.values(queryModules);
  await expect(loadQuery()).resolves.toBe("?raw");
});
