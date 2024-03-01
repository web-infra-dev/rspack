it("should pass query to loader without resource", function () {
	var result = require("./loaders/queryloader?query!");
	expect(result).toEqual({
		query: "?query",
		// CHANGE: 'prev' property is not supported by Rspack at the moment
		// prev: null
	});
});
