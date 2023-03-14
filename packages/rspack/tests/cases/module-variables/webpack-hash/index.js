it("__webpack_hash__", function () {
	expect(typeof __webpack_hash__).toBe("string");
	expect(__webpack_hash__.length > 0).toBe(true);
});
