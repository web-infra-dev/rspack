const prod = __MODE__ === "production";

it("should allow to create css modules", done => {
	prod
		? __non_webpack_require__("./use-style_js.bundle1.js")
		: __non_webpack_require__("./use-style_js.bundle0.js");
	import("./use-style.js").then(({ default: x }) => {
		try {
			expect(x).toMatchFileSnapshot(`${__SNAPSHOT__}/${__STATS_I__}_${prod ? "prod" : "dev"}.txt`);
		} catch (e) {
			return done(e);
		}
		done();
	}, done);
});

it("should allow to process tailwind as global css", done => {
	prod
		? __non_webpack_require__("./tailwind_min_css.bundle1.js")
		: __non_webpack_require__("./tailwind_min_css.bundle0.js");
	import("./tailwind.min.css").then(() => done(), done);
});
