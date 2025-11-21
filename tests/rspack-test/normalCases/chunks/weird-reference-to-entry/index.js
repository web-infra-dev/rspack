it("should handle reference to entry chunk correctly", async function() {
	await import(/* webpackChunkName: "main" */"./module-a").then(function(result) {
		expect(result.default).toBe("ok");
	});
});
