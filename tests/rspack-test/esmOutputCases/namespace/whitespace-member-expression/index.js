import * as foo from "./foo.js";

const value = foo .yar();

export default value;

it("should preserve namespace member access with whitespace around the dot", async () => {
  const mod = await import(/* webpackIgnore: true */ "./main.mjs");

  expect(mod.default).toBe("yar?");
});
