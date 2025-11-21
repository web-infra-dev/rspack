it("ensure that require context is created correctly", function () {
	const pageObjectContext = require.context("./folder", true, /-story$/);
	expect(pageObjectContext(`./my-story`)).toBeDefined();
	expect(() => {
		pageObjectContext(`./my-other-story`);
	}).toThrowError(/Cannot find module/);
});
