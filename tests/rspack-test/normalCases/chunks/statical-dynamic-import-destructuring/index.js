import { _ as swcAsyncToGenerator } from "@swc/helpers/_/_async_to_generator";

function _async_to_generator(generator) {
	return async function (...args) {
		const iterator = generator.apply(this, args);
		const yielded = iterator.next();
		const namespace = await yielded.value;
		expect(namespace.a).toBe(1);
		expect(namespace.usedExports).toBe(true);
		return iterator.next(namespace).value;
	};
}

it("should load only used exports", async () => {
	const { default: def, usedExports } = await import("../statical-dynamic-import/dir1/a");
	expect(def).toBe(3);
	expect(usedExports).toEqual(["default", "usedExports"]);
});

it("should get warning on using 'webpackExports' with destructuring assignment", async () => {
	const { default: def } = await import(/* webpackExports: ["a"] */"../statical-dynamic-import/dir1/a?2");
	expect(def).toBe(3);
});

it("should not tree-shake default export for exportsType=default module", async () => {
	const { default: object } = await import("../statical-dynamic-import/dir2/json/object.json");
	const { default: array } = await import("../statical-dynamic-import/dir2/json/array.json");
	const { default: primitive } = await import("../statical-dynamic-import/dir2/json/primitive.json");
	expect(object).toEqual({ a: 1 });
	expect(array).toEqual(["a"]);
	expect(primitive).toBe("a");
	const { default: a } = await import("../statical-dynamic-import/dir2/a");
	expect(a).toEqual({ a: 1, b: 2 });
});

it("should not tree-shake default export for exportsType=default context module", async () => {
	const dir = "json";
	const { default: object } = await import(`../statical-dynamic-import/dir3/${dir}/object.json`);
	const { default: array } = await import(`../statical-dynamic-import/dir3/${dir}/array.json`);
	const { default: primitive } = await import(`../statical-dynamic-import/dir3/${dir}/primitive.json`);
	expect(object).toEqual({ a: 1 });
	expect(array).toEqual(["a"]);
	expect(primitive).toBe("a");
	const file = "a";
	const { default: a } = await import(`../statical-dynamic-import/dir3/${file}`);
	expect(a).toEqual({ a: 1, b: 2 });
});

it("should static analyze dynamic import variable destructuring assignment", async () => {
	const m = await import("../statical-dynamic-import/dir1/a?3");
	const { default: def, usedExports } = m;
	expect(def).toBe(3);
	expect(usedExports).toEqual(["default", "usedExports"]);
});

it("expect support of \"deep\" tree-shaking for destructuring assignment dynamic import", async () => {
	const { a: { aaa, usedExports: usedExportsA }, b: { bbb, usedExports: usedExportsB } } = await import("./lib");
	expect(aaa).toBe(1);
	expect(bbb).toBe(2);
	expect(usedExportsA).toEqual(["aaa", "usedExports"]);
	expect(usedExportsB).toEqual(["bbb", "usedExports"]);
});

it("should preserve a dynamic import namespace yielded before destructuring", async () => {
	function* load() {
		const { default: def } = yield import("../statical-dynamic-import/dir1/a?yield");
		return def;
	}

	const iterator = load();
	const namespace = await iterator.next().value;
	expect(namespace.a).toBe(1);
	expect(namespace.default).toBe(3);
	expect(namespace.usedExports).toBe(true);
	expect(iterator.next({ default: 42 }).value).toBe(42);
});

it("should tree shake a dynamic import awaited through SWC's async helper", async () => {
	const load = swcAsyncToGenerator(function* () {
		const { default: def, usedExports } = yield import("../statical-dynamic-import/dir1/a?swc-yield");
		return { def, usedExports };
	});

	const { def, usedExports } = await load();
	expect(def).toBe(3);
	expect(usedExports).toEqual(["default", "usedExports"]);
});

it("should keep nested generators conservative inside SWC's async helper", async () => {
	const load = swcAsyncToGenerator(function* () {
		function* loadDestructured() {
			const { default: def } = yield import("../statical-dynamic-import/dir1/a?swc-nested-destructuring");
			return def;
		}

		function* loadMember() {
			return (yield import("../statical-dynamic-import/dir1/a?swc-nested-member")).a;
		}

		const destructuredIterator = loadDestructured();
		const destructuredNamespace = yield destructuredIterator.next().value;
		expect(destructuredNamespace.a).toBe(1);
		expect(destructuredNamespace.usedExports).toBe(true);

		const memberIterator = loadMember();
		const memberNamespace = yield memberIterator.next().value;
		expect(memberNamespace.default).toBe(3);
		expect(memberNamespace.usedExports).toBe(true);

		return [
			destructuredIterator.next({ default: 42 }).value,
			memberIterator.next({ a: 43 }).value
		];
	});

	expect(await load()).toEqual([42, 43]);
});

it("should not trust an arbitrary helper with the same name", async () => {
	const load = _async_to_generator(function* () {
		const { default: def } = yield import("../statical-dynamic-import/dir1/a?fake-swc-yield");
		return def;
	});

	expect(await load()).toBe(3);
});
