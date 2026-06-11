const fs = require("fs");

it("should prefer rspack-prefixed magic comments over webpack-prefixed ones", async () => {
	const { default: value } = await import(
		/* webpackChunkName: "webpack-prefix-ignored", rspackChunkName: "rspack-prefix-wins" */ "./conflict"
	);

	expect(value).toBe("conflict");
	const files = fs.readdirSync(__dirname);
	expect(files).toContain("rspack-prefix-wins.js");
	expect(files).not.toContain("webpack-prefix-ignored.js");
});

it("should keep the first magic comment when the same prefixed comment repeats", async () => {
	const { default: value } = await import(
		/* webpackChunkName: "first-duplicate-wins", webpackChunkName: "second-duplicate-ignored" */ "./duplicate"
	);

	expect(value).toBe("duplicate");
	const files = fs.readdirSync(__dirname);
	expect(files).toContain("first-duplicate-wins.js");
	expect(files).not.toContain("second-duplicate-ignored.js");
});
