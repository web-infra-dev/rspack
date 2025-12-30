import * as reexport from "./reexport.js"

it("should handle cjs reexport default from fake namespace", () => {
  expect(reexport.lib.default).toEqual({
    a: 1,
    b: 2,
    "default": 3,
  });
  expect(reexport.lib.default.a).toEqual(1);
  expect(reexport.lib.default.b).toEqual(2);
  expect(reexport.lib.default.default).toEqual(3);
  expect(reexport.lib.default.c).toBeUndefined();
});

it("should handle cjs reexport from fake namespace", () => {
  expect(reexport.lib).toMatchObject({
    a: 1,
    b: 2,
    "default": {
      a: 1,
      b: 2,
      "default": 3
    }
  });
  expect(reexport.lib.a).toEqual(1);
  expect(reexport.lib.b).toEqual(2);
  expect(reexport.lib.c).toBeUndefined();
});