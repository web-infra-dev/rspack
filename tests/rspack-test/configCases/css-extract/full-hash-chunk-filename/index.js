it("should define the full-hash runtime for an extracted async CSS filename", async () => {
	await expect(
		import(/* webpackChunkName: "async" */ "./async")
	).resolves.toMatchObject({ value: 1 });

	const fs = require("fs");
	const assets = fs.readdirSync(__STATS__.outputPath);
	const runtime = fs.readFileSync(
		`${__STATS__.outputPath}/main.js`,
		"utf-8"
	);

	expect(assets.some(name => /^async\.[a-f0-9]+\.css$/.test(name))).toBe(true);
	expect(runtime).toContain("miniCssF");
	expect(runtime).toContain("__webpack_require__.h()");
	expect(runtime).toMatch(/__webpack_require__\.h\s*=/);
});
