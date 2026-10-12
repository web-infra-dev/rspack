it("should preserve lazy loader maps and their mutations across native loaders", () => {
	for (const value of [
		require("./fixture.js?unread"),
		require("./fixture.js?drop"),
		require("./fixture.js?read"),
		require("./fixture.js?noop"),
		require("./fixture.js?stringify"),
		require("./fixture.js?freeze"),
		require("./fixture.js?field"),
		require("./fixture.js?array"),
		require("./fixture.js?descriptor"),
		require("./fixture.js?delete"),
		require("./fixture.js?replace")
	]) {
		expect(value).toBe(42);
	}
});
