it("should not parse new URL(import.meta.url) to generate a dynamic chunk", () => {
  const path = __non_webpack_require__("path");
  const dir = path.dirname(new URL(import.meta.url).pathname);
  expect(dir).toEqual(__TEST_SOURCE_PATH__);
})