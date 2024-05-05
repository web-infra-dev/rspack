it("should parsed into const dependency", function () {
	const mock = "http://www.mock.com";
	__webpack_public_path__ = __webpack_base_uri__ = mock;
	expect(__webpack_public_path__).toBe(mock);
	expect(__webpack_base_uri__).toBe(mock);
});
