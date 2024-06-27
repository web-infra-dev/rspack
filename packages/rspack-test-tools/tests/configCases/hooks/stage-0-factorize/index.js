const fs = require("fs");
require('./a')
const path = require("path");
require('./b')

it("should not call factorize hook for requiest: 'fs' and 'path' (bailed by external)", () => {
	const mainFile = fs.readFileSync(path.normalize(__filename), "utf-8");
	expect(
		mainFile.startsWith(
`
/* ./index.js */
/* ./b */
/* ./a */
`.trim()
		)
	).toBeTruthy();
});
