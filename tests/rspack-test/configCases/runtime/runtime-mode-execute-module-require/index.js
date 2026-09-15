import value from "./loader!./source";

it("passes rspack require into executeModule", () => {
  expect(value).toBe(42);
});
