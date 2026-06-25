const prod = __MODE__ === "production";

it("should allow to create css modules", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	prod
		? require("./use-style_js.bundle1.js")
		: require("./use-style_js.bundle0.js");
	import("./use-style.js").then(({ default: x }) => {
		try {
			expect(x).toMatchFileSnapshotSync(`${__SNAPSHOT__}/${__STATS_I__}_${prod ? "prod" : "dev"}.txt`);
		} catch (e) {
			return done(e);
		}
		done();
	}, done);
}));

it("should allow to process tailwind as global css", async() => {
	prod
		? require("./tailwind_min_css.bundle1.js")
		: require("./tailwind_min_css.bundle0.js");
	await import("./tailwind.min.css");
});
