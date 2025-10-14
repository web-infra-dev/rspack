it("should accept itself and pass data", (done) => {
	require("./file")(done);
	NEXT(require("@rspack/test-tools/helper/legacy/update")(done));
});
