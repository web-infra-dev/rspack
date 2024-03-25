it("global false", function () {
	global;
	expect(__webpack_require__.g).toBe(undefined);
});
