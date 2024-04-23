global.getNumberTwo = function () {
	return 2;
};
it("should success exec function from before.js", () => {
	expect(getNumberOne()).toBe(1);
});
