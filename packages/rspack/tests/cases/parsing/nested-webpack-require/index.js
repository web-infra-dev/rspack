function __webpack_require__() {}
__webpack_require__.n = 1;

it("__nested_webpack_require_", function () {
	expect(typeof __webpack_require__).toBe("function");
	expect(__webpack_require__.n).toBe(1);
});
