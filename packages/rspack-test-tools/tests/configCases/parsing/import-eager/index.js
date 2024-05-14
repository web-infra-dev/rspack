const fs = require("fs")

it("should be able to use import eager", async function () {
	const { default: a } = await import(/* webpackMode: "eager" */"./two");
	expect(a).toBe(2);
	const { default: b } = await import(/* webpackMode: "eager" */`./two`);
	expect(b).toBe(2);
	const files = await fs.promises.readdir(__dirname);
	expect(files.filter(f => f.endsWith(".js")).length).toBe(1)
});
