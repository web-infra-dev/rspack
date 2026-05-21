import directDefault from "./default-literal.js";
import * as referencedDefault from "./default-reference.js";

const fs = __non_webpack_require__("fs");
const generated = /** @type {string} */ (fs.readFileSync(__filename, "utf-8"));

it("should inline default exports", () => {
	// START:A
	expect(directDefault).toBe(4);
	expect(referencedDefault.default).toBe(3);
	// END:A
	const block = generated.match(/\/\/ START:A([\s\S]*)\/\/ END:A/)[1];
	expect(block.includes(`((/* inlined export ["default"] */4)).toBe(4)`)).toBe(true);
	expect(block.includes(`((/* inlined export ["default"] */3)).toBe(3)`)).toBe(true);
});
