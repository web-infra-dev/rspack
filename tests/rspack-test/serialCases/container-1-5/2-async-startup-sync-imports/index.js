const fs = require("fs");
const path = require("path");

// Reset federation state between serial cases to ensure deterministic share scopes.
if (globalThis.__FEDERATION__) {
	globalThis.__GLOBAL_LOADING_REMOTE_ENTRY__ = {};
	//@ts-ignore
	globalThis.__FEDERATION__.__INSTANCES__.forEach(instance => {
		instance.moduleCache.clear();
		if (globalThis[instance.name]) {
			delete globalThis[instance.name];
		}
	});
	globalThis.__FEDERATION__.__INSTANCES__ = [];
}

it("should load the component from container", () => {
	return import("./App").then(({ default: App }) => {
		const rendered = App();
		const initial = parseRenderVersions(rendered);
		expect(initial.host).toBe("2.1.0");
		expect(initial.localB).toBe("2.1.0");
		expect(["0.1.2", "3.2.1"]).toContain(initial.remote);
		return import("./upgrade-react").then(({ default: upgrade }) => {
			upgrade();
			const rendered = App();
			const upgraded = parseRenderVersions(rendered);
			expect(upgraded.host).toBe("3.2.1");
			expect(upgraded.localB).toBe("3.2.1");
			expect(["0.1.2", "3.2.1"]).toContain(upgraded.remote);
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
const parseRenderVersions = rendered => {
	const match = rendered.match(
		/^App rendered with \[This is react ([^\]]+)\] and \[ComponentA rendered with \[This is react ([^\]]+)\]\] and \[ComponentB rendered with \[This is react ([^\]]+)\]\]$/
	);
	if (!match) {
		throw new Error(`Unexpected render output: ${rendered}`);
	}
	return {
		host: match[1],
		remote: match[2],
		localB: match[3]
	};
};
