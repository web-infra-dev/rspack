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

it("should not have duplicate async wrappers in CommonJS bundle", () => {
	const baseDir = __dirname.endsWith("module") ? path.dirname(__dirname) : __dirname;
	const content = fs.readFileSync(path.join(baseDir, "main.js"), "utf-8");

	// Count occurrences of Promise.resolve().then wrapper pattern
	const wrapperMatches = content.match(/Promise\.resolve\(\)\.then\(function\(\)\s*\{/g) || [];
	expect(wrapperMatches.length).toBe(1); // Should only have ONE async wrapper

	// Should NOT have nested Promise wrappers
	expect(content).not.toMatch(/Promise\.resolve\(\)\.then\([^)]*Promise\.resolve\(\)\.then/);
});

it("should not have duplicate async wrappers in ESM bundle", () => {
	const baseDir = __dirname.endsWith("module") ? path.dirname(__dirname) : __dirname;
	const content = fs.readFileSync(path.join(baseDir, "module", "main.mjs"), "utf-8");

	// Count occurrences of async wrapper pattern
	const wrapperMatches = content.match(/const __webpack_exports__Promise = Promise\.resolve\(\)\.then\(/g) || [];
	expect(wrapperMatches.length).toBe(1); // Should only have ONE async wrapper
});

it("should set ASYNC_FEDERATION_STARTUP runtime global in CommonJS bundle", () => {
	const baseDir = __dirname.endsWith("module") ? path.dirname(__dirname) : __dirname;
	const content = fs.readFileSync(path.join(baseDir, "main.js"), "utf-8");

	// Should have ASYNC_FEDERATION_STARTUP flag set to prevent duplicate wrappers
	expect(content).toMatch(/__webpack_require__\.asf\s*=|ASYNC_FEDERATION_STARTUP/);
});

it("should have startup entrypoint runtime requirement in CommonJS bundle", () => {
	const baseDir = __dirname.endsWith("module") ? path.dirname(__dirname) : __dirname;
	const content = fs.readFileSync(path.join(baseDir, "main.js"), "utf-8");

	// With async startup, should use startup entrypoint (not regular startup)
	expect(content).toMatch(/__webpack_require__\.X|STARTUP_ENTRYPOINT/);
});

it("container entry should not have async wrapper", () => {
	const baseDir = __dirname.endsWith("module") ? path.dirname(__dirname) : __dirname;
	const containerContent = fs.readFileSync(path.join(baseDir, "container.js"), "utf-8");

	// Container entry (library export) should NOT have async wrapper
	// It needs to be synchronously available for consumption
	const hasAsyncWrapper = /Promise\.resolve\(\)\.then\(function\(\)\s*\{/.test(containerContent);
	expect(hasAsyncWrapper).toBe(false);
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
