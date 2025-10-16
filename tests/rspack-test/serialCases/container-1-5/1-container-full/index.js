const fs = require("fs");
const path = require("path");

it("should load the component from container", () => {
	return import("./App").then(({ default: App }) => {
		const rendered = App();
		expect(rendered).toBe(
			"App rendered with [This is react 2.1.0] and [ComponentA rendered with [This is react 2.1.0]] and [ComponentB rendered with [This is react 2.1.0]]"
		);
		return import("./upgrade-react").then(({ default: upgrade }) => {
			upgrade();
			const rendered = App();
			expect(rendered).toBe(
				"App rendered with [This is react 3.2.1] and [ComponentA rendered with [This is react 3.2.1]] and [ComponentB rendered with [This is react 3.2.1]]"
			);
		});
	});
});

it("should emit promise-based bootstrap in CommonJS bundle", () => {
	const content = fs.readFileSync(path.join(__dirname, "main.js"), "utf-8");
	expect(content).toContain("Promise.resolve().then(function() {");
});

it("should emit awaited bootstrap in ESM bundle", () => {
	const content = fs.readFileSync(
		path.join(__dirname, "module", "main.mjs"),
		"utf-8"
	);
	expect(content).toContain(
		"const __webpack_exports__Promise = Promise.resolve().then(async () =>"
	);
	expect(content).toContain("export default await __webpack_exports__Promise;");
});
