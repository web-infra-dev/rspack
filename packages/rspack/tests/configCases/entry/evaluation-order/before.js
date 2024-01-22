it("should exec fn which defined `index.js` success", () => {
	globalThis.getNumberOne = function () {
		return 1;
	};
	expect(getNumberTwo()).toBe(2);
});
