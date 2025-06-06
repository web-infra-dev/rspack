export default 'vendor2'

it('should not contain vendor1 vendor2 in file', () => {
	const path = __non_webpack_require__('path')
	const fs = __non_webpack_require__('fs')

	const modules = fs.readFileSync(path.resolve(__dirname, './main.js'), 'utf-8')
	expect(modules).toMatchInlineSnapshot(`"use strict";
(self["webpackChunk"] = self["webpackChunk"] || []).push([["main"], {
"./main.js": (function (__unused_webpack_module, __unused_webpack___webpack_exports__, __webpack_require__) {
/* ESM import */var _vendor1__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__("./vendor1.js");
/* ESM import */var _vendor2__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__("./vendor2.js");



__webpack_require__.e(/* import() */ "async_js").then(__webpack_require__.bind(__webpack_require__, "./async.js"))

it('should not contain vendor1 and vendor2 in current chunk', (done) => {
	expect(_vendor1__WEBPACK_IMPORTED_MODULE_1__/* ["default"] */.Z).toBe('vendor1')
	expect(_vendor2__WEBPACK_IMPORTED_MODULE_0__/* ["default"] */.Z).toBe('vendor2')
})


}),

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
__webpack_require__.O(0, ["vendor2",], function() {
        return __webpack_exec__("./main.js");
      });
var __webpack_exports__ = __webpack_require__.O();

}
]);`)
})
