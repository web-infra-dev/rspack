it("should add all modules headers info above modules", () => {
  const fs = require("fs");
  const path = require("path")
  const content = fs.readFileSync(path.join(__dirname, "sut.js"), "utf-8");

  const runtimeRequirements = content.includes("__rspack_context.r")
    ? "__rspack_context.r, __rspack_context.n, __rspack_context"
    : "__webpack_require__.n, __webpack_require__, __webpack_require__";
  expect(content).toContain(`
/*!****************!*\\
  !*** ./sut.js ***!
  \\****************/
/*! namespace exports */
/*! runtime requirements: ${runtimeRequirements} */
/*! Statement with side_effects in source code at ./sut.js:3:1-29 */
    `.trim())

  expect(content).toContain(`
/*!****************!*\\
  !*** ./cjs.js ***!
  \\****************/
/*! unknown exports (runtime-defined) */
/*! runtime requirements: module */
    `.trim())
})
