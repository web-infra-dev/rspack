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

it("should strip a resource BOM for normal loaders without a source map", () => {
  expect(require("./resource.txt?resource-plain-normal")).toBe("hello\n");
  expect(require("./resource.txt?resource-extract-normal")).toBe("hello\n");
});

it("should preserve resource BOM bytes for raw loaders without a source map", () => {
  expect(require("./resource.txt?resource-plain-raw")).toBe("efbbbf68656c6c6f0a");
  expect(require("./resource.txt?resource-extract-raw")).toBe("efbbbf68656c6c6f0a");
});

it("should strip a resource BOM for normal loaders after extracting a source map", () => {
  expect(require("./resource-with-map.txt?resource-extract-normal")).toBe("hello\n");
});

it("should preserve resource BOM bytes for raw loaders after extracting a source map", () => {
  expect(require("./resource-with-map.txt?resource-extract-raw")).toBe("efbbbf68656c6c6f0a");
});
