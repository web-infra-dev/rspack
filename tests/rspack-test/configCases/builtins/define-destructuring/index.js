const fs = require("fs");
const path = require("path");

const { NODE_ENV, DEEP: { A } } = ENV;
const {
	REPEATED: { LEFT },
	REPEATED: { RIGHT }
} = ENV;
const {
	COMPLETE: { A: completeA },
	COMPLETE: complete
} = ENV;

it("destructures nested define objects and prunes the rest", () => {
	expect(NODE_ENV).toBe("production");
	expect(A).toBe(1);
	expect(LEFT).toBe(3);
	expect(RIGHT).toBe(4);
	expect(completeA).toBe(5);
	expect(complete).toEqual({ A: 5, B: 6 });

	const bundle = fs.readFileSync(path.join(__dirname, "./bundle0.js"), "utf-8");
	expect(bundle).toContain('"NODE_ENV":"production"');
	expect(bundle).toContain('"A":1');
	// Split the literals so this file itself does not contain them contiguously.
	expect(bundle).not.toContain("DEB" + "UG");
	expect(bundle).not.toContain('"B"' + ":2");
});
