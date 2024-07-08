it("should exec fn which defined `index.js` success", () => {
	global.getNumberOne = function () {
		return 1;
	};
	expect(getNumberTwo()).toBe(2);
});
