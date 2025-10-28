it("should handle var injection in require.ensure with error callback", () => new Promise(done => {
	require.ensure([], function(require) {
		require("./empty");
		var x = module.x;
		done();
	}, function(error) {}, "chunk-with-var-inject");
}));
