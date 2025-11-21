import * as values from "./const"

it("should generate correct export for dynamic reexports (dynamic cjs)", () => {
  values;
  expect(values.value).toBe(42);
})
