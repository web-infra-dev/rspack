import * as ns from "./module";

it("should preserve member order across inline and spilled chains", () => {
	expect(ns.tree.a["b"].value).toBe(42);
	expect(ns.tree.a.b.read()).toBe(42);
	expect(ns.build().a.b.value).toBe(42);
	expect(require("./module").tree.a.b.value).toBe(42);
	expect(typeof ns.tree.a.b.value).toBe("number");
});

it("should stop at dynamic keys and preserve optional member boundaries", () => {
	let calls = 0;
	const key = () => {
		calls++;
		return "a";
	};
	expect(ns.tree[key()].b.value).toBe(42);
	expect(calls).toBe(1);
	expect(ns.tree?.a.b?.read()).toBe(42);
	expect(ns.missing?.[key()].b.value).toBeUndefined();
	expect(calls).toBe(1);
});

it("should preserve the non-optional prefix of awaited imports", async () => {
	expect((await import("./module")).tree.a["b"].value).toBe(42);
	expect((await import("./module")).tree?.a.b.read()).toBe(42);
	expect((await import("./module")).missing?.a.b.value).toBeUndefined();
});
