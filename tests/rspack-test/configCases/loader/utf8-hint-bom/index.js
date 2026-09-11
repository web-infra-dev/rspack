it("should preserve a string BOM across loader boundaries", () => {
  expect(require("./input.js?string-normal")).toBe("\ufeffhello");
});

it("should strip a Buffer BOM when converting to a string", () => {
  expect(require("./input.js?buffer-normal")).toBe("hello");
});

it("should preserve BOM bytes for raw loaders", () => {
  expect(require("./input.js?string-raw")).toBe("efbbbf68656c6c6f");
  expect(require("./input.js?buffer-raw")).toBe("efbbbf68656c6c6f");
});
