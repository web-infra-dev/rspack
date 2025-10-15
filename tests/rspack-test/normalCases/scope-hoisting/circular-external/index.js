it("should compile and run fine", async function() {
	await Promise.all([
		import("./a1"),
		import("./b1"),
		import("./c1")
	]);
});
