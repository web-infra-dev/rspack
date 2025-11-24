const fs = require("fs");
const path = require("path");

// Reset federation state between serial cases to avoid leaking share scopes/runtime data.
if (globalThis.__FEDERATION__) {
	globalThis.__GLOBAL_LOADING_REMOTE_ENTRY__ = {};
	// @ts-ignore
	globalThis.__FEDERATION__.__INSTANCES__.forEach(instance => {
		instance.moduleCache.clear();
		if (globalThis[instance.name]) {
			delete globalThis[instance.name];
		}
	});
	// @ts-ignore
	globalThis.__FEDERATION__.__INSTANCES__ = [];
}

const isESM = () =>
	__dirname.includes("/module") || __dirname.includes("\\module");
const runtimeFile = () =>
	path.join(__dirname, `runtime.${isESM() ? "mjs" : "js"}`);

const parseRendered = rendered =>
	rendered.match(
		/^App rendered with \[This is react ([^\]]+)\] and \[ComponentA rendered with \[This is react ([^\]]+)\]\] and \[ComponentB rendered with \[This is react ([^\]]+)\]\]$/
	);

it("should load the component from container", () => {
	return import("./App").then(({ default: App }) => {
		const rendered = App();
		const match = parseRendered(rendered);
		expect(match).toBeTruthy();
		// Host + local should start on 2.1.0
		expect(match[1]).toBe("2.1.0");
		expect(match[3]).toBe("2.1.0");
		// Remote may keep its own version when not shared.
		expect(["0.1.2", "2.1.0"]).toContain(match[2]);

		return import("./upgrade-react").then(({ default: upgrade }) => {
			upgrade();
			const after = App();
			const upgraded = parseRendered(after);
			expect(upgraded).toBeTruthy();
			// Host and local component should upgrade.
			expect(upgraded[1]).toBe("3.2.1");
			expect(["2.1.0", "3.2.1"]).toContain(upgraded[3]);
			// Remote may keep its own version or upgrade if shared.
			expect(["0.1.2", "2.1.0", "3.2.1"]).toContain(upgraded[2]);
		});
	});
});

it("should wire async startup through the shared runtime chunk", () => {
	const content = fs.readFileSync(runtimeFile(), "utf-8");

	// Guard added to make calling STARTUP() without args safe when async startup is enabled.
	expect(content).toContain("__webpack_require__startup");
	expect(content).toContain("chunkIds === undefined && result === undefined");

	// Ensure async federation runtime hook is present in the runtime chunk.
	expect(content).toContain("__webpack_require__mf_startup_once");
});
