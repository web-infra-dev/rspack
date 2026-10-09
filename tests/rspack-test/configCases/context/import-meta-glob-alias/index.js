const modules = import.meta.glob(
  ["@/glob-imports/*.js", "!@/glob-imports/excluded.js"],
  { eager: true },
);
const ancestorExcludedModules = import.meta.glob(
  ["@/glob-imports/*.js", "!@/**/excluded.js"],
  { eager: true },
);
const caseInsensitiveModules = import.meta.glob("@/glob-imports/*.JS", {
  caseSensitive: false,
  eager: true,
});
const mixedRelativeModules = import.meta.glob(
  ["@/glob-imports/*.js", "./local/*.js"],
  { eager: true },
);
const mixedAliasModules = import.meta.glob(
  ["@/glob-imports/*.js", "~/local/*.js"],
  { eager: true },
);
const metacharacterDirectoryModules = import.meta.glob("special-dir/*.js", {
  eager: true,
});

it("resolves an alias in import.meta.glob and honors exclusions", () => {
  expect(Object.keys(modules)).toEqual(["/src/glob-imports/value.js"]);
  expect(modules["/src/glob-imports/value.js"].default).toBe("resolved");
});

it("applies exclusions rooted above the resolved glob directory", () => {
  expect(Object.keys(ancestorExcludedModules)).toEqual([
    "/src/glob-imports/value.js",
  ]);
});

it("resolves an alias with case-insensitive matching", () => {
  expect(Object.keys(caseInsensitiveModules)).toEqual([
    "/src/glob-imports/excluded.js",
    "/src/glob-imports/value.js",
  ]);
});

it("resolves aliases mixed with relative patterns", () => {
  expect(Object.keys(mixedRelativeModules)).toEqual([
    "./local/value.js",
    "/src/glob-imports/excluded.js",
    "/src/glob-imports/value.js",
  ]);
});

it("resolves each alias in a multi-pattern glob", () => {
  expect(Object.keys(mixedAliasModules)).toEqual([
    "/local/value.js",
    "/src/glob-imports/excluded.js",
    "/src/glob-imports/value.js",
  ]);
});

it("treats metacharacters in an alias target as literal path characters", () => {
  expect(Object.keys(metacharacterDirectoryModules)).toEqual([
    "/src/[dir]/value.js",
  ]);
});

it("keeps bare patterns without an alias relative to the importer", async () => {
  const { nestedModules, queryModules } = await import("./src/bare/globs.js");
  expect(Object.values(nestedModules)).toEqual(["nested"]);
  const [loadQuery] = Object.values(queryModules);
  await expect(loadQuery()).resolves.toBe("?raw");
});

it("keeps empty alias entries and explicit relative patterns local", async () => {
  const { emptyAliasModules, explicitRelativeModules, mjsQueryModules } =
    await import("./src/bare/globs.js");
  expect(emptyAliasModules).toEqual({ "./dir/value.js": "local-dir" });
  expect(explicitRelativeModules).toEqual({
    "./@/glob-imports/value.js": "literal-at",
  });
  expect(Object.keys(mjsQueryModules)).toEqual(["./value.mjs"]);
  await expect(mjsQueryModules["./value.mjs"]()).resolves.toBe("?raw");
});

it("matches aliased directory names without case sensitivity", () => {
  const modules = import.meta.glob("@/GLOB-IMPORTS/*.JS", {
    caseSensitive: false,
    eager: true,
    import: "default",
  });
  expect(modules).toEqual({
    "/src/glob-imports/excluded.js": "excluded",
    "/src/glob-imports/value.js": "resolved",
  });
});

it("supports ignored aliases without dropping other patterns", () => {
  expect(import.meta.glob("ignored/*.js", { eager: true })).toEqual({});
  const modules = import.meta.glob(["ignored/*.js", "./local/*.js"], {
    eager: true,
    import: "default",
  });
  expect(modules).toEqual({ "./local/value.js": "local" });
});

const escapedRootModules = import.meta.glob(
  ["@meta/*.js", "!@meta/excluded.js"],
  { eager: true },
);
it("keeps metacharacters in resolved alias directories literal", () => {
  expect(Object.keys(escapedRootModules)).toEqual([
    "/src/[aliased]/{assets}/value.js",
  ]);
  expect(Object.values(escapedRootModules)[0].default).toBe("resolved");
});

const relocatedModules = import.meta.glob(
  ["@relocated/*.js", "!@relocated/excluded.js"],
  { eager: true },
);
it("uses remaining patterns when afterResolve relocates an alias scan root", () => {
  expect(Object.keys(relocatedModules)).toEqual([
    "/src/[aliased]/{assets}/value.js",
  ]);
  expect(Object.values(relocatedModules)[0].default).toBe("resolved");
});

it("excludes local patterns when scanning case-insensitive aliased directories", () => {
  const modules = import.meta.glob(
    ["@/GLOB-IMPORTS/*.JS", "./LOCAL/*.JS", "!./local/value.js", "!@/**/excluded.js"],
    { caseSensitive: false, eager: true, import: "default" },
  );
  expect(modules).toEqual({ "/src/glob-imports/value.js": "resolved" });
});
