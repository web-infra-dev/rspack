it("__webpack_modules__", function () {
	expect(__webpack_modules__).not.toBeUndefined();
	__webpack_modules__.a = 1;
	expect(__webpack_modules__.a).toBe(1);
});
