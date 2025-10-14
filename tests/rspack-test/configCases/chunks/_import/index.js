it("should be able to use import", async function () {
	await import("./two")
		.then(function (two) {
			expect(two.default).toEqual(2);
		});
});
