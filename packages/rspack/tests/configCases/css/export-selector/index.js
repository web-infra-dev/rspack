const fs = require('fs')
const path = require('path')

it('should have correct css result', async () => {
	const css = await fs.promises.readFile(path.resolve(__dirname, './imported_js.bundle0.css'))
	expect(css.toString()).toMatchSnapshot()
})

it("should allow to dynamic import a css module", done => {
	import("./style.module.css").then(x => {
		try {
			expect(x).toEqual(
				nsObj({
					foo: "foo",
					bar: "b a r",
					dashName: "dashName",
					local: "local",
				})
			);
		} catch (e) {
			return done(e);
		}
		done();
	}, done);
});

it("should allow to reexport a css module", done => {
	__non_webpack_require__("./reexported_js.bundle0.js");
	import("./reexported").then(x => {
		try {
			expect(x).toEqual(
				nsObj({
					foo: "foo",
					bar: "b a r",
					dashName: "dashName",
					local: "local",
				})
			);
		} catch (e) {
			return done(e);
		}
		done();
	}, done);
});

it("should allow to import a css module", done => {
	__non_webpack_require__("./imported_js.bundle0.js");
	import("./imported").then(({ default: x }) => {
		try {
			expect(x).toEqual(
				nsObj({
					foo: "foo",
					bar: "b a r",
					dashName: "dashName",
					local: "local",
				})
			);
		} catch (e) {
			return done(e);
		}
		done();
	}, done);
});
