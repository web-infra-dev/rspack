const fs = require("fs");
const path = require("path");

it("should load chunk with patched chunk handler", () => {
	return import("./App").then(({ default: App }) => {
		const rendered = App();
		console.log(rendered)
		expect(rendered).toBe(
			"App fetched with Chunk Handler PASS"
		);
	});
});

it("should emit promise-based bootstrap in global script bundle", () => {
	const content = fs.readFileSync(path.join(__dirname, "main.js"), "utf-8");
	expect(content).toContain("Promise.resolve().then(function() {");
});
