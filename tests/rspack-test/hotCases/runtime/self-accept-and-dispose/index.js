it("should accept itself and pass data", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	require("./file")(done);
	NEXT(require("@rspack/test-tools/helper/legacy/update")(done));
}));
