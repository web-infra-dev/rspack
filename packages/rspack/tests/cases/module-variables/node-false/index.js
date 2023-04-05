it("should not load node bindings when node option is false", function () {
	global;
	expect(__webpack_require__.g).toBe(undefined);
});
