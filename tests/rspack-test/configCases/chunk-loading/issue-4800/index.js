it("import with template string should work well", async () => {
	await import(`./logo1`).then(res => {
		expect(res.default).toBe("logo1");
	});
});

it("require with template string should work well", () => {
	const res = require(`./logo2`);
	expect(res).toBe("logo2");
});
