it("should load only used exports", async () => {
	const def = (await import("../statical-dynamic-import/dir1/a")).default;
	const usedExports = (await import("../statical-dynamic-import/dir1/a")).usedExports;
	expect(def).toBe(3);
	expect(usedExports).toEqual(["default", "usedExports"]);
});

it("should get warning on using 'webpackExports' with statical dynamic import", async () => {
	const def = (await import(/* webpackExports: ["a"] */"../statical-dynamic-import/dir1/a?2")).default;
	expect(def).toBe(3);
});

it("should not tree-shake default export for exportsType=default module", async () => {
	const object = (await import("../statical-dynamic-import/dir2/json/object.json")).default;
	const array = (await import("../statical-dynamic-import/dir2/json/array.json")).default;
	const primitive = (await import("../statical-dynamic-import/dir2/json/primitive.json")).default;
	expect(object).toEqual({ a: 1 });
	expect(array).toEqual(["a"]);
	expect(primitive).toBe("a");
	const a = (await import("../statical-dynamic-import/dir2/a")).default;
	expect(a).toEqual({ a: 1, b: 2 });
});

it("should not tree-shake default export for exportsType=default context module", async () => {
	const dir = "json";
	const object = (await import(`../statical-dynamic-import/dir3/${dir}/object.json`)).default;
	const array = (await import(`../statical-dynamic-import/dir3/${dir}/array.json`)).default;
	const primitive = (await import(`../statical-dynamic-import/dir3/${dir}/primitive.json`)).default;
	expect(object).toEqual({ a: 1 });
	expect(array).toEqual(["a"]);
	expect(primitive).toBe("a");
	const file = "a";
	const a = (await import(`../statical-dynamic-import/dir3/${file}`)).default;
	expect(a).toEqual({ a: 1, b: 2 });
});

it("expect support of \"deep\" tree-shaking for followed members dynamic import", async () => {
	const aaa = (await import("./lib")).a.aaa;
	const usedExportsA = (await import("./lib")).a.usedExports;
	const bbb = (await import("./lib")).b.bbb;
	const usedExportsB = (await import("./lib")).b.usedExports;
	expect(aaa).toBe(1);
	expect(bbb).toBe(2);
	expect(usedExportsA).toEqual(["aaa", "usedExports"]);
	expect(usedExportsB).toEqual(["bbb", "usedExports"]);
});

it("should analyze args for members call", async () => {
	const res = (await import("./lib2")).a.inc((await import("./lib2")).b.aaa);
	expect(res).toBe(2);
	expect((await import("./lib2")).a.usedExports).toEqual(["inc", "usedExports"]);
	expect((await import("./lib2")).b.usedExports).toEqual(["aaa", "usedExports"]);
});
