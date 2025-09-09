const fs = require("fs");
require('./a')
const path = require("path");
require('./b')

it("should not call factorize hook for requiest: 'fs' and 'path' (bailed by external)", () => {
	const mainFile = fs.readFileSync(path.normalize(__filename), "utf-8");
	const requests = mainFile.match(/\/\*: .* :\*\//g);
	const wrap = (request) => `/*: ${request} :*/`;
	expect(requests).toContain(wrap('./index.js'));
	expect(requests).toContain(wrap('./a'));
	expect(requests).toContain(wrap('./b'));
	expect(requests).not.toContain(wrap('fs'));
	expect(requests).not.toContain(wrap('path'));
});
