const prod = process.env.NODE_ENV === "production";

it("should allow to create css modules", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	prod
		? __non_webpack_require__("./340.bundle0.js")
		: __non_webpack_require__("./use-style_js.bundle0.js");
	import("./use-style.js").then(({ default: x }) => {
		try {
			expect(x).toMatchSnapshot(`${__SNAPSHOT__}/${prod ? "prod" : "dev"}.txt`);
		} catch (e) {
			return done(e);
		}
		done();
	}, done);
}));
