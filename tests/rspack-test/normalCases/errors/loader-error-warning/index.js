it("should emit the correct errors and warnings", function() {
	require("./error-loader.mjs?abc!./a");
	require("./error-loader.mjs?def!./a");
	require("./warning-loader.mjs?xyz!./a");
});
