it("global true", function () {
	global;
	expect(__webpack_require__.g).not.toBe("undefined");
});
