const fs = require("fs");
const path = require("path");

it("should reject weak import without loading an async chunk", async function () {
	await expect(
		import(/* webpackMode: "weak" */ "./weak-dependency")
	).rejects.toMatchObject({
		code: "MODULE_NOT_FOUND",
		message: expect.stringMatching(/weak dependency/)
	});

	const jsFiles = fs
		.readdirSync(__dirname)
		.filter(file => file.endsWith(".js") || file.endsWith(".mjs"));
	expect(jsFiles).toEqual(["bundle0.js"]);

	const source = fs.readFileSync(path.join(__dirname, "bundle0.js"), "utf-8");
	expect(source).not.toMatch(/__webpack_require__\.e\(\s*\/\* import\(\) \*\//);
	expect(source).toContain("weak dependency");
});
