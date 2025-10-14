it("should accept itself and pass data", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	require("./file")(done);
	NEXT(require("../../update")(done));
}));
