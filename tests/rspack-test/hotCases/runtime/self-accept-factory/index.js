it("should able to accept for another module", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	require("./a")(done);
	NEXT(require("@rspack/test-tools/helper/legacy/update")(done));
}));
