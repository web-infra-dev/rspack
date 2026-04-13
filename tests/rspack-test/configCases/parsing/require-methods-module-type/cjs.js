it("should enable require.context in javascript/dynamic", function () {
	expect(require.context("./dir", false, /\.js$/)("./a.js")).toBe("a");
});

it("should enable require.ensure in javascript/dynamic", () => new Promise((resolve, reject) => {
	require.ensure(["./ensure"], function (require) {
		try {
			expect(require("./ensure")).toBe("ensure");
			resolve();
		} catch (error) {
			reject(error);
		}
	});
}));
