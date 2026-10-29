import directValue from "dir/value.js";

const modules = import.meta.glob("dir/*.js", {
  eager: true,
  import: "default",
});

const literalModules = import.meta.glob("dir/value.js", {
  eager: true,
  import: "default",
});

const excludedModules = import.meta.glob(["./dir/*.js", "!dir/*.js"], {
  eager: true,
  import: "default",
});

it("keeps trailing-slash exact aliases from matching glob suffixes", () => {
  expect(directValue).toBe("local");
  expect(modules).toEqual({ "./dir/value.js": "local" });
  expect(literalModules).toEqual({ "./dir/value.js": "local" });
  expect(excludedModules).toEqual({});
});
