const fs = require("fs");

it("should emit source with harmony eval", async () => {
	const source = fs.readFileSync(__dirname + "/main.js", "utf-8");

	expect(source).toMatch(/eval\(.+\/\/# sourceURL=webpack/);

	expect(source).toMatch(
		"//# sourceURL=webpack://blackalbum/./index.scss?steins_gaess=god&css|"
	);
});
