const context = require.context("./modules", false, /\.js$/);
const eagerContext = require.context("./modules", false, /\.js$/, "eager");
const weakContext = require.context("./modules", false, /\.js$/, "weak");

it("should link a synchronous context through local initializers", () => {
	expect(context.keys().sort()).toEqual(["./a.js", "./b.js"]);
	expect(context.resolve("./a.js")).toContain("./modules/a.js");
	expect(context("./a.js").value).toBe("a");
	expect(context("./b.js").value).toBe("b");
});

it("should preserve resolve semantics for eager and weak contexts", async () => {
	expect(await eagerContext.resolve("./a.js")).toContain("./modules/a.js");
	expect((await eagerContext("./a.js")).value).toBe("a");
	expect(weakContext.resolve("./b.js")).toContain("./modules/b.js");
	expect(weakContext("./b.js").value).toBe("b");
});
