const prod = process.env.NODE_ENV === "production";

it("should allow to create css modules", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	const chunk = prod
		? require("fs")
				.readdirSync(__dirname)
				.find(file => /^\d+\.bundle0\.js$/.test(file))
		: null;
	prod
		? eval("require")(`./${chunk}`)
		: require("./use-style_js.bundle0.js");
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
