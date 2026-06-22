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
/*! unknown exports (runtime-defined) */
/*! runtime requirements: module */
/*! Statement with side_effects in source code at ./cjs.js:1:1-3:2 */    
    `.trim())

  if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
    expect(content).toContain(`
/*!*****************!*\\
  !*** ./util.js ***!
  \\*****************/
/*! namespace exports */
/*! export default [not provided] [unused] [provision prevents renaming] */
/*! export message [provided] [used in sut] [inlined to ("hello")] */
/*! export secret [maybe provided (runtime-defined)] [used in sut] [provision prevents renaming] -> ./cjs.js secret */
/*! other exports [maybe provided (runtime-defined)] [unused] -> ./cjs.js */
/*! runtime requirements: __rspack_exports, __rspack_context.r, __rspack_context.o, __rspack_context.n, __rspack_context.d, __rspack_context */
`.trim())
  } else {
    expect(content).toContain(`
/*!*****************!*\\
  !*** ./util.js ***!
  \\*****************/
/*! namespace exports */
/*! export default [not provided] [unused] [provision prevents renaming] */
/*! export message [provided] [used in sut] [inlined to ("hello")] */
/*! export secret [maybe provided (runtime-defined)] [used in sut] [provision prevents renaming] -> ./cjs.js secret */
/*! other exports [maybe provided (runtime-defined)] [unused] -> ./cjs.js */
/*! runtime requirements: __webpack_require__.o, __webpack_require__.n, __webpack_require__.d, __webpack_require__, __webpack_require__, __webpack_exports__ */
`.trim())
  }



})
