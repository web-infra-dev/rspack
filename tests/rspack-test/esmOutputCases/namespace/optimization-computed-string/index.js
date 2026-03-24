import * as foo from "./foo.js";

export const a = foo["bar"]["quux"]["a"]();
export const b = foo["bar"]["quux"]["b"]();

it("should optimize namespace lookup chains for computed string keys", async () => {
  const mod = await import(/* webpackIgnore: true */ "./main.mjs");

  expect(mod.a).toBe("effect");
  expect(mod.b).toBe("b");
});
