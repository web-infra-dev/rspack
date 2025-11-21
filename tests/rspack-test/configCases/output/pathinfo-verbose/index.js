it("should add all modules headers info above modules", () => {
    const fs = require("fs");
    const path = require("path")
    const content = fs.readFileSync(path.join(__dirname, "sut.js"), "utf-8");

    expect(content).toContain(`
/*!****************!*\\
  !*** ./sut.js ***!
  \\****************/
/*! namespace exports */
/*! runtime requirements: __webpack_require__ */
/*! Statement with side_effects in source code at ./sut.js:3:0-28 */    
    `.trim())

    expect(content).toContain(`
/*!****************!*\\
  !*** ./cjs.js ***!
  \\****************/
/*! unknown exports (runtime-defined) */
/*! runtime requirements: module */
/*! Statement with side_effects in source code at ./cjs.js:1:0-3:1 */    
    `.trim())

    expect(content).toContain(`
/*!*****************!*\\
  !*** ./util.js ***!
  \\*****************/
/*! namespace exports */
/*! export default [not provided] [unused] [provision prevents renaming] */
/*! export message [provided] [used in sut] [provision prevents renaming] */
/*! export secret [maybe provided (runtime-defined)] [used in sut] [provision prevents renaming] -> ./cjs.js secret */
/*! other exports [maybe provided (runtime-defined)] [unused] -> ./cjs.js */
/*! runtime requirements: __webpack_require__.o, __webpack_require__.n, __webpack_require__.d, __webpack_require__.*, __webpack_require__, __webpack_exports__ */
`.trim())



})