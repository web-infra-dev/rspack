const fs = require("fs");
const path = require("path");

it("should narrow an awaited namespace through a destructured alias", async () => {
  let {z: alias} = await import(/* webpackChunkName: "narrowed" */ "./barrel");
  expect(alias.used()).toBe(137);
  const content = fs.readFileSync(path.join(__dirname, STRICT_IMPORT_RECEIVER ? "strict" : "loose", "narrowed.js"), "utf8");
  if (!STRICT_IMPORT_RECEIVER) {
    expect(content).not.toContain("NESTED_NAMESPACE_UNUSED_DROP_20d2c7");
  }
});
it("should trace callback, Promise.all and computed literal bindings", async () => {
  expect(await import("./barrel").then(({z}) => z.used())).toBe(137);
  expect(await import("./barrel").then(function({z}) { return z.used(); })).toBe(137);
  const [{z}] = await Promise.all([import("./barrel")]);
  expect(z.used()).toBe(137);
  const {["z"]: other} = await import("./barrel");
  expect(other.used()).toBe(137);
  const {z: {used: direct}} = await import("./barrel");
  expect(direct()).toBe(137);
});
it("should retain the namespace when the alias escapes", async () => {
  const {z} = await import("./escaped-barrel");
  expect(Object.keys(z).sort()).toEqual(["unused", "used"]);
  expect(z.unused).toBe("nested-namespace-escape-kept");
});
it("should retain rest properties and callback arguments", async () => {
  const {z: {...rest}} = await import("./escaped-barrel");
  expect(rest.unused).toBe("nested-namespace-escape-kept");
  expect(await import("./escaped-barrel").then(function({z}) {
    return [z.used, arguments[0].z.unused];
  })).toEqual([139, "nested-namespace-escape-kept"]);
});
it("should preserve destructuring getter reads even for unused bindings", async () => {
  globalThis.__nestedDestructureGetter = 0;
  const {z: {unused}} = await import("./getter");
  expect(globalThis.__nestedDestructureGetter).toBe(1);
});
it("should keep defaults that replace the imported value", async () => {
  const {z = {value: 149, other: 151}} = await import("./fallback");
  expect(Object.values(z)).toEqual([149,151]);
});
it("should preserve direct eval and reassigned bindings", async () => {
  expect(await import("./eval").then(m => m.read())).toBe("nested-namespace-escape-kept");
  let {z: other} = await import("./escaped-barrel");
  other = {unused: "reassigned"};
  expect(other.unused).toBe("reassigned");
});
it("should retain values exported under another name", async () => {
  const {renamed} = await import("./exported");
  expect(renamed.unused).toBe("nested-namespace-escape-kept");
  const {renamed: whole} = await import("./exported-whole");
  expect(whole.z.unused).toBe("nested-namespace-escape-kept");
});
it("should retain the receiver of array prototype calls", async () => {
  const {values} = await import("./array");
  expect(values.map(n => n + 1)).toEqual([158,164,168]);
});
