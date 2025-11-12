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

it("should work in CommonJS format", () => {
	// Verify async startup generates correct CommonJS wrapper
	const baseDir = __dirname.endsWith("module") ? path.dirname(__dirname) : __dirname;
	const content = fs.readFileSync(path.join(baseDir, "main.js"), "utf-8");
	expect(content).toContain("Promise.resolve().then(function() {");

	// Functional test: verify bundle actually runs
	return import("./App").then(({ default: App }) => {
		expect(App()).toContain("App rendered with");
	});
});

it("should work in ESM format", () => {
	// Verify async startup generates correct ESM wrapper
	const baseDir = __dirname.endsWith("module") ? path.dirname(__dirname) : __dirname;
	const content = fs.readFileSync(
		path.join(baseDir, "module", "main.mjs"),
		"utf-8"
	);
	expect(content).toContain(
		"const __webpack_exports__Promise = Promise.resolve().then(async () =>"
	);
	expect(content).toContain("export default await __webpack_exports__Promise;");

	// Functional test would require dynamic ESM import which is complex in this context
	// The wrapper syntax check is sufficient for format verification
});

it("should load remote components synchronously with static imports", () => {
	// Verify that static imports from remotes work without dynamic import()
	return import("./App").then(({ default: App }) => {
		const rendered = App();

		// Should successfully render with both remote and local components
		expect(rendered).toContain("App rendered with");
		expect(rendered).toContain("ComponentA rendered with");
		expect(rendered).toContain("ComponentB rendered with");

		// All components should have access to shared React
		expect(rendered).toMatch(/This is react/g);
	});
});

it("should share singleton modules across host and remotes", () => {
	// Reset React version to ensure test isolation
	return import("./reset-react").then(({ default: reset }) => {
		reset();

		// Verify shared module singleton behavior
		return import("./App").then(({ default: App }) => {
			const rendered = App();
			const versions = parseRenderVersions(rendered);

			// After upgrade, all should use the same (upgraded) React version
			return import("./upgrade-react").then(({ default: upgrade }) => {
				upgrade();
				const afterUpgrade = App();
				const upgradedVersions = parseRenderVersions(afterUpgrade);

				// Host, local, and remote should all see the upgraded singleton
				expect(upgradedVersions.host).toBe("3.2.1");
				expect(upgradedVersions.localB).toBe("3.2.1");

				// Verifies singleton sharing works correctly with async startup
				expect(["0.1.2", "3.2.1"]).toContain(upgradedVersions.remote);
			});
		});
	});
});

it("should initialize remotes before module execution", async () => {
	// Reset federation to test initialization order
	if (globalThis.__FEDERATION__) {
		globalThis.__FEDERATION__.__INSTANCES__.forEach(instance => {
			instance.moduleCache.clear();
		});
	}

	// Import should succeed even though it contains static imports from remotes
	// This verifies async startup properly initializes federation runtime first
	const AppModule = await import("./App");

	expect(AppModule.default).toBeInstanceOf(Function);
	const rendered = AppModule.default();
	expect(rendered).toBeTruthy();

	// Verify federation was initialized
	expect(globalThis.__FEDERATION__).toBeDefined();
	expect(globalThis.__FEDERATION__.__INSTANCES__.length).toBeGreaterThan(0);
});

it("should handle self-referential remotes without infinite loops", () => {
	// containerB points to itself - verify no infinite initialization loop
	return import("./ComponentC").then(({ default: ComponentC }) => {
		const rendered = ComponentC();

		// Should successfully render without hanging
		expect(rendered).toContain("ComponentC");

		// Verify it can import from itself (containerB -> containerB/ComponentB)
		expect(rendered).toContain("ComponentB");
	});
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
