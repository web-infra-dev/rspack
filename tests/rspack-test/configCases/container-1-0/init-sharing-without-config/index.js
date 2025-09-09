it("should have single runtime chunk", () => {
	__webpack_init_sharing__("default");
	expect(typeof __webpack_require__.I).toBe("function")
});
