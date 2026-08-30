it("should add all modules headers info above modules", () => {
  const fs = require("fs");
  const path = require("path")
  const content = fs.readFileSync(path.join(__dirname, "sut.js"), "utf-8");

  if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
    expect(content).toContain(`
/*!****************!*\\
  !*** ./sut.js ***!
  \\****************/
/*! namespace exports */
/*! runtime requirements: __rspack_context.r, __rspack_context */
/*! Statement with side_effects in source code at ./sut.js:3:1-29 */
    `.trim())
  } else {
    expect(content).toContain(`
/*!****************!*\\
  !*** ./sut.js ***!
  \\****************/
/*! namespace exports */
/*! runtime requirements: __webpack_require__, __webpack_require__ */
/*! Statement with side_effects in source code at ./sut.js:3:1-29 */
    `.trim())
  }

  expect(content).toContain(`
/*!****************!*\\
  !*** ./cjs.js ***!
  \\****************/
/*! default exports */
/*! export secret [provided] [used in sut] [renamed to r] */
/*! runtime requirements: module */
/*! Statement with side_effects in source code at ./cjs.js:1:1-3:2 */
    `.trim())

  if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
    expect(content).toContain(`
/*!*****************!*\\
  !*** ./util.js ***!
  \\*****************/
/*! namespace exports */
/*! export message [provided] [used in sut] [inlined to ("hello")] */
/*! export secret [provided] [used in sut] [renamed to r] -> ./cjs.js secret */
/*! runtime requirements: __rspack_exports, __rspack_context.r, __rspack_context.n, __rspack_context.d, __rspack_context */
`.trim())
  } else {
    expect(content).toContain(`
/*!*****************!*\\
  !*** ./util.js ***!
  \\*****************/
/*! namespace exports */
/*! export message [provided] [used in sut] [inlined to ("hello")] */
/*! export secret [provided] [used in sut] [renamed to r] -> ./cjs.js secret */
/*! runtime requirements: __webpack_require__.n, __webpack_require__.d, __webpack_require__, __webpack_require__, __webpack_exports__ */
`.trim())
  }



})
