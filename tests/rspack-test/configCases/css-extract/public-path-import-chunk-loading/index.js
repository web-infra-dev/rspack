export function loadFoo() {
  return import("./foo");
}

it("should inject public path runtime for async extracted css with import chunk loading", () => {
  const fs = __non_webpack_require__("fs");
  const source = fs.readFileSync(`${__STATS__.outputPath}/main.js`, "utf-8");

  expect(typeof loadFoo).toBe("function");
  expect(source).toContain("__webpack_require__.e");
  expect(source).toContain("__webpack_require__.f.miniCss");
  expect(source).toContain('__webpack_require__.p = "/"');
  expect(source).toContain("var fullhref = __webpack_require__.p + href");
});
