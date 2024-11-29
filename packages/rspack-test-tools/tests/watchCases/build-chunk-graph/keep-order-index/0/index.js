it('should not panic', async () => {
	await Promise.all([
		import("./ab.js"),
		import("./ba.js"),
	]);
	expect(1).toBe(1)
})
