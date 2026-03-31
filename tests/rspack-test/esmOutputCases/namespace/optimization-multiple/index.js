import * as foo from "./foo.js";

function a() {
  return [foo.foo(), foo.foo()];
}

export default a().join(",");

it("should optimize repeated namespace lookups in one function", async () => {
  const mod = await import(/* webpackIgnore: true */ "./main.mjs");

  expect(mod.default).toBe("foo,foo");
});
