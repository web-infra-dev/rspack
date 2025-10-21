const fs = require("fs");
const path = require("path");

it("should load the component from container", () => {
	return import("./App").then(({ default: App }) => {
		const rendered = App();
		// CJS build (main.js) gets react 3.2.1 from previous test (1-container-full)
		// ESM build (module/main.mjs) starts fresh with react 2.1.0
		const isESM = __dirname.endsWith("module");
		const initialVersion = isESM ? "2.1.0" : "3.2.1";
		const upgradedVersion = isESM ? "3.2.1" : "4.3.2";

		expect(rendered).toBe(
			`App rendered with [This is react ${initialVersion}] and [ComponentA rendered with [This is react ${initialVersion}]] and [ComponentB rendered with [This is react ${initialVersion}]]`
		);
		return import("./upgrade-react").then(({ default: upgrade }) => {
			upgrade();
			const rendered = App();
			expect(rendered).toBe(
				`App rendered with [This is react ${upgradedVersion}] and [ComponentA rendered with [This is react ${upgradedVersion}]] and [ComponentB rendered with [This is react ${upgradedVersion}]]`
			);
		});
	});
});

it("should emit promise-based bootstrap in CommonJS bundle", () => {
	// Determine the base directory (handling both CJS and ESM execution contexts)
	const baseDir = __dirname.endsWith("module") ? path.dirname(__dirname) : __dirname;
	const content = fs.readFileSync(path.join(baseDir, "main.js"), "utf-8");
	expect(content).toContain("Promise.resolve().then(function() {");
});

it("should emit awaited bootstrap in ESM bundle", () => {
	// Determine the base directory (handling both CJS and ESM execution contexts)
	const baseDir = __dirname.endsWith("module") ? path.dirname(__dirname) : __dirname;
	const content = fs.readFileSync(
		path.join(baseDir, "module", "main.mjs"),
		"utf-8"
	);
	expect(content).toContain(
		"const __webpack_exports__Promise = Promise.resolve().then(async () =>"
	);
	expect(content).toContain("export default await __webpack_exports__Promise;");
});
