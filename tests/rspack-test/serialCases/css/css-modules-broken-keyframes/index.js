const prod = process.env.NODE_ENV === "production";

it("should allow to create css modules", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	const chunk = prod
		? __non_webpack_require__("fs")
				.readdirSync(__dirname)
				.find(file => /^\d+\.bundle0\.js$/.test(file))
		: null;
	prod
		? __non_webpack_require__(`./${chunk}`)
		: __non_webpack_require__("./use-style_js.bundle0.js");
	import("./use-style.js").then(({ default: x }) => {
		try {
			expect(x).toMatchFileSnapshotSync(
				`${__SNAPSHOT__}/${prod ? "prod" : "dev"}.txt`
			);
		} catch (e) {
			return done(e);
		}
		done();
	}, done);
}));
