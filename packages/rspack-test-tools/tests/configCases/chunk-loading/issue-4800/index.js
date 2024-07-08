it("import with template string should work well", done => {
	import(`./logo1`).then(res => {
		expect(res.default).toBe("logo1");
		done();
	});
});

it("require with template string should work well", () => {
	const res = require(`./logo2`);
	expect(res).toBe("logo2");
});
