export { a } from "./common.js";
export { c } from "./other.js";

it("should allow reexporting bindings from another entry", async () => {
  const main = await import(/* webpackIgnore: true */ "./main.mjs");
  const other = await import(/* webpackIgnore: true */ "./other.mjs");

  expect(typeof main.a).toBe("object");
  expect(main.c).toBe(other.c);
  expect(typeof other.b).toBe("object");
});
