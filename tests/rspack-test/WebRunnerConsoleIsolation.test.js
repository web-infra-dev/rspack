const fs = require("node:fs");
const path = require("node:path");
const { WebRunner } = require("@rspack/test-tools");

it("keeps concurrent web-runner console overrides isolated", async () => {
	const root = path.resolve(__dirname, "js/web-runner-console-isolation");
	const bundle = path.join(root, "bundle.js");
	fs.mkdirSync(root, { recursive: true });
	fs.writeFileSync(bundle, "module.exports = console;\n");

	const createRunner = name =>
		new WebRunner({
			location: "https://test.cases/path/index.html",
			env: { expect, it, beforeEach, afterEach },
			name,
			testConfig: {},
			source: root,
			dist: root,
			compilerOptions: { target: "web", node: false },
			runInNewContext: true
		});

	const originalWarn = console.warn;
	try {
		const first = await createRunner("first").run(bundle);
		const second = await createRunner("second").run(bundle);
		const firstWarnings = [];
		const secondWarnings = [];
		first.warn = warning => firstWarnings.push(warning);
		second.warn = warning => secondWarnings.push(warning);

		first.warn("first warning");
		second.warn("second warning");

		expect(first).not.toBe(console);
		expect(second).not.toBe(console);
		expect(first).not.toBe(second);
		expect(firstWarnings).toEqual(["first warning"]);
		expect(secondWarnings).toEqual(["second warning"]);
		expect(console.warn).toBe(originalWarn);
	} finally {
		console.warn = originalWarn;
		fs.rmSync(root, { recursive: true, force: true });
	}
});
