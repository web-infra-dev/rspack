import "./weak-present.js";

it("should remove unused exports from an empty destructured lazy import", async () => {
	const {} = await import("./pure.js");
});

it("should evaluate a lazy import without retaining unused exports", async () => {
	globalThis.__rspack_empty_destructuring_lazy__ = 0;
	const {} = await import("./effectful.js");
	expect(globalThis.__rspack_empty_destructuring_lazy__).toBe(1);
	delete globalThis.__rspack_empty_destructuring_lazy__;
});

it("should evaluate an eager import without retaining unused exports", async () => {
	globalThis.__rspack_empty_destructuring_eager__ = 0;
	const {} = await import(/* webpackMode: "eager" */ "./eager.js");
	expect(globalThis.__rspack_empty_destructuring_eager__).toBe(1);
	delete globalThis.__rspack_empty_destructuring_eager__;
});

it("should evaluate context targets without retaining unused exports", async () => {
	globalThis.__rspack_empty_destructuring_context__ = 0;
	async function load(name) {
		const {} = await import(`./context/${name}.js`);
	}
	await load("pure");
	await load("effectful");
	expect(globalThis.__rspack_empty_destructuring_context__).toBe(1);
	delete globalThis.__rspack_empty_destructuring_context__;
});

it("should keep evaluation of a dynamically imported side-effects-free module", async () => {
	globalThis.__rspack_empty_destructuring_declared__ = 0;
	const {} = await import("./declared-side-effects-free.js");
	expect(globalThis.__rspack_empty_destructuring_declared__).toBe(1);
	delete globalThis.__rspack_empty_destructuring_declared__;
});

it("should preserve a then export and its namespace receiver", async () => {
	const {} = await import("./then.js");
	expect(globalThis.__rspack_empty_destructuring_then__).toBe("then-receiver-kept");
	delete globalThis.__rspack_empty_destructuring_then__;
});

it("should remove unused exports from an available weak import", async () => {
	const {} = await import(/* webpackMode: "weak" */ "./weak-present.js");
	expect(globalThis.__rspack_empty_destructuring_weak__).toBe(1);
	delete globalThis.__rspack_empty_destructuring_weak__;
});

it("should reject an unavailable weak import", async () => {
	await expect((async () => {
		const {} = await import(/* webpackMode: "weak" */ "./weak-missing.js");
	})()).rejects.toThrow(/weak dependency/);
});

it("should not collide empty-destructured and full-namespace context imports", async () => {
	async function preload(name) {
		const {} = await import(`./mixed-context/${name}.js`);
	}
	async function load(name) {
		return import(`./mixed-context/${name}.js`);
	}
	await preload("target");
	const key = "kept";
	const mod = await load("target");
	expect(mod[key]).toBe("MIXED_CONTEXT_KEPT");
});

it("should preserve default export for destructured source-phase imports", async () => {
	const {} = await import.source("./source-phase.js");
	const { value } = await import.source("./source-phase-prop.js");
	expect(value).toBe("source-prop-kept");
});

it("should remove unused exports from an unused dynamic import binding", async () => {
	globalThis.__rspack_empty_destructuring_unused_binding__ = 0;
	const _unused = await import("./unused-binding.js");
	expect(globalThis.__rspack_empty_destructuring_unused_binding__).toBe(1);
	delete globalThis.__rspack_empty_destructuring_unused_binding__;
});

it("should remove unused exports from an empty destructured then callback", async () => {
	globalThis.__rspack_empty_destructuring_then_cb__ = 0;
	await import("./then-empty.js").then(({}) => {});
	expect(globalThis.__rspack_empty_destructuring_then_cb__).toBe(1);
	delete globalThis.__rspack_empty_destructuring_then_cb__;
});
