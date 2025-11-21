const lib = require("./lib");
const fs = require('fs')
const path = require('path')

it("minify-plugin", () => {
	expect(lib.answer).toEqual(42);
	const dirs = fs.readdirSync(path.resolve(__dirname, './'));
	expect(dirs.some(dirname => dirname.endsWith('.map')))
});
