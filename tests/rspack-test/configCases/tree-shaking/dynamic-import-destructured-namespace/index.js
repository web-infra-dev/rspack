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
it("should index existing and fallback references across overlapping binding prefixes", async () => {
  const {z: {used}, z: unusedParent, z: {used: unusedDuplicate}} =
    await import(/* webpackChunkName: "prefixes" */ "./prefix-barrel");
  expect(used).toBe(179);
  const {z: {used: unusedLeaf}, z: fallbackParent} =
    await import(/* webpackChunkName: "prefixes" */ "./prefix-barrel");
  const content = fs.readFileSync(path.join(__dirname, STRICT_IMPORT_RECEIVER ? "strict" : "loose", "prefixes.js"), "utf8");
  expect(content).not.toContain("OVERLAPPING_PREFIX_UNUSED_DROP");
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

it("should retain exported namespaces with deconflicted binding names", async () => {
  const modules = await Promise.all([
    import("./reserved-require"),
    import("./reserved-exports"),
    import("./reserved-context"),
    import("./reserved-rspack-exports"),
    import("./export-before-declaration")
  ]);
  for (const module of modules) {
    expect(module.renamed.used).toBe(139);
    expect(module.renamed.unused).toBe("nested-namespace-escape-kept");
  }
});
it("should preserve calls, typeof and shorthand references after deconfliction", async () => {
  const modules = await Promise.all([
    import("./call-require"),
    import("./call-exports"),
    import("./call-context"),
    import("./call-rspack-exports")
  ]);
  const names = ["__webpack_require__", "__webpack_exports__", "__rspack_context", "__rspack_exports"];
  for (let i = 0; i < modules.length; i++) {
    expect(modules[i].result).toBe(157);
    expect(modules[i].type).toBe("object");
    expect(modules[i].wrapped[names[i]].unused).toBe("called-namespace-kept");
  }
});
it("should distinguish a shadowed local from an exported binding", async () => {
  const module = await import("./shadowed-export");
  expect(module.alias).toBe("unrelated");
  expect(await module.read()).toBe(137);
  if (!STRICT_IMPORT_RECEIVER) {
    const content = fs.readFileSync(path.join(__dirname, "loose", "shadowed.js"), "utf8");
    expect(content).not.toContain("SHADOWED_NAMESPACE_UNUSED_DROP");
  }
});

it("should preserve shorthand declarations and callback bindings", async () => {
  const module = await import("./reserved-shorthand");
  expect(module.values).toEqual([157,157,157,157]);
  for (const value of Object.values(module.wrapped)) {
    expect(value.unused).toBe("called-namespace-kept");
  }
  const called = await import("./reserved-shorthand-barrel").then(({
    __webpack_require__, __webpack_exports__, __rspack_context, __rspack_exports
  }) => [__webpack_require__.used(), __webpack_exports__.used(), __rspack_context.used(), __rspack_exports.used()]);
  expect(called).toEqual([157,157,157,157]);
  const [{__webpack_require__, __webpack_exports__, __rspack_context, __rspack_exports}] =
    await Promise.all([import("./reserved-shorthand-barrel")]);
  expect([__webpack_require__.used(), __webpack_exports__.used(), __rspack_context.used(), __rspack_exports.used()])
    .toEqual([157,157,157,157]);
});
