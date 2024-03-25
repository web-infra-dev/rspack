it("__webpack_public_path__", function () {
	expect(__webpack_public_path__).toBe("/");
	__webpack_public_path__ = "/a";
	expect(__webpack_public_path__).toBe("/a");
	const a = __webpack_public_path__;
	expect(a).toBe("/a");
	expect(__webpack_require__.p).toBe("/a");
});

it("__webpack_public_path__ use as local varable", function () {
	var __webpack_public_path__ = "/test";
	// __webpack_require__.p set by prev test
	expect(__webpack_require__.p).toBe("/a");
	expect(__webpack_public_path__).toBe("/test");
});
