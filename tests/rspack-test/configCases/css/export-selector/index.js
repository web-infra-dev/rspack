const fs = require('fs')
const path = require('path')

it('should have correct css result', async () => {
	const css = await fs.promises.readFile(path.resolve(eval('__dirname'), './imported_js.bundle0.css'))
	expect(css.toString()).toMatchFileSnapshotSync(path.join(__SNAPSHOT__, 'imported_js.bundle0.css.txt'));
})

it("should allow to dynamic import a css module", async () => {
	await import("./style.module.css").then(x => {
			expect(x).toEqual(
				nsObj({
					foo: "foo",
					bar: "b a r",
					dashName: "dashName",
					local: "local",
				})
			);
	});
});

it("should allow to reexport a css module", async () => {
	require("./reexported_js.bundle0.js");
	await import("./reexported").then(x => {
			expect(x).toEqual(
				nsObj({
					foo: "foo",
					bar: "b a r",
					dashName: "dashName",
					local: "local",
				})
			);
	});
});

it("should allow to import a css module", async () => {
	require("./imported_js.bundle0.js");
	await import("./imported").then(({ default: x }) => {
			expect(x).toEqual(
				nsObj({
					foo: "foo",
					bar: "b a r",
					dashName: "dashName",
					local: "local",
				})
			);
	});
});
