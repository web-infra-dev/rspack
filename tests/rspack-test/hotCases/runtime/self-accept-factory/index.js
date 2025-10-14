it("should able to accept for another module", (done) => {
	require("./a")(done);
	NEXT(require("@rspack/test-tools/helper/legacy/update")(done));
});
