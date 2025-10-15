it("should compile and work", () => new Promise(done => {
	require.ensure(
		["./foo"],
		() => {
			throw new Error("error");
		},
		() => {
			import("./foo").then(m => {
				expect(m.default).toBe("foo");
				done();
			});
		}
	);
}));
