"use strict";
it("should include only one use strict per module", function () {
	require("./harmony-with-strict");
	require("./harmony-without-strict");
	require("./harmony-with-strict2");

	var fs = require("fs");
	var source = fs.readFileSync(__filename, "utf-8");

	var regExp = /\"use strict\";?\s*(.*)/g;
	var match = regExp.exec(source);
	var matches = [];
	while (match) {
		matches.push(match[1]);
		match = regExp.exec(source);
	}

	matches.sort();

	expect(matches).toHaveLength(1);
	expect(matches[0]).toMatch(/^var __(?:rspack|webpack)_modules(?:__)? = \(\{$/);

});
