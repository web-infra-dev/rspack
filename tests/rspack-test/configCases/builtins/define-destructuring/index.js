const fs = require("fs");
const path = require("path");

const { NODE_ENV, DEEP: { A } } = ENV;

it("destructures nested define objects and prunes the rest", () => {
	expect(NODE_ENV).toBe("production");
	expect(A).toBe(1);

	const bundle = fs.readFileSync(path.join(__dirname, "./bundle0.js"), "utf-8");
	expect(bundle).toContain('"NODE_ENV":"production"');
	expect(bundle).toContain('"A":1');
	// Split the literals so this file itself does not contain them contiguously.
	expect(bundle).not.toContain("DEB" + "UG");
	expect(bundle).not.toContain('"B"' + ":2");
});
