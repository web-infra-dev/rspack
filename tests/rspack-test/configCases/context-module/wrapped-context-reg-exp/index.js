it("should not include foo.js", async () => {
	let a1 = 'a1';
	let a2 = 'a2';
	expect(require('./sub/' + a1)).toBe("a1");
	expect(() => require('./sub/' + a2)).toThrow();
});
