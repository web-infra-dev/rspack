import x from "./module";

it("should have correct this context", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	expect(x).toEqual("ok1");

	(function() {
		import.meta.webpackHot.accept("./module", () => {
			expect(x).toEqual("ok2");
			expect(this).toEqual({ ok: true });
			done();
		});
	}).call({ ok: true });

	NEXT(require("../../update")(done));
}));
