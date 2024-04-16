// clean federation globals from previous executions
if (globalThis.__FEDERATION__) {
	globalThis.__GLOBAL_LOADING_REMOTE_ENTRY__ = {};
	//@ts-ignore
	globalThis.__FEDERATION__.__INSTANCES__.map((i) => {
		i.moduleCache.clear();
		if (globalThis[i.name]) {
			delete globalThis[i.name];
		}
	});
	globalThis.__FEDERATION__.__INSTANCES__ = [];
}

let warnings = [];
let oldWarn;

beforeEach(done => {
	oldWarn = console.warn;
	console.warn = m => warnings.push(m);
	done();
});

afterEach(done => {
	expectWarning();
	console.warn = oldWarn;
	done();
});

const expectWarning = (regexp, index) => {
	if (!regexp) {
		expect(warnings).toEqual([]);
	} else {
		expect(warnings[index]).toMatch(regexp);
	}
	warnings.length = 0;
};

it("should load the component from container", () => {
	return import("./App").then(({ default: App }) => {
		expectWarning(
			/Version 8 from main of shared singleton module react does not satisfy the requirement of main which needs \^2/,
			1
		);
		const rendered = App();
		expect(rendered).toBe(
			"App rendered with [This is react 8] and [This is react 2.1.0] and [This is react 8] and [ComponentC rendered with [This is react 8] and [ComponentA rendered with [This is react 8]] and [ComponentB rendered with [This is react 8]]]"
		);
		return import("./upgrade-react").then(({ default: upgrade }) => {
			upgrade();
			const rendered = App();
			expect(rendered).toBe(
				"App rendered with [This is react 9] and [This is react 2.1.0] and [This is react 9] and [ComponentC rendered with [This is react 9] and [ComponentA rendered with [This is react 9]] and [ComponentB rendered with [This is react 9]]]"
			);
		});
	});
});

import Self from "./Self";

it("should load itself from its own container", () => {
	return import("self/Self").then(({ default: RemoteSelf }) => {
		expect(RemoteSelf).toBe(Self);
	});
});
