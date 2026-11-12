import value from "./loader.mjs!./source";

it("passes rspack require into executeModule", () => {
  expect(value).toBe(42);
});
