const fs = require("fs");

it("should emit source with harmony eval", async () => {
	const source = fs.readFileSync(__dirname + "/main.js", "utf-8");

	// eval() with sourceURL
	expect(source).toMatch(/eval\(.+\/\/# sourceURL=webpack/);

	// devtoolNamespace and devtoolModuleFilenameTemplate
	expect(source).toMatch(
		"//# sourceURL=" + encodeURI("webpack://blackalbum/./index.scss?steins_gaess=god&css|")
	);
});
