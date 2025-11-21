const fs = require("fs");

var a = 'a';
var b = 'b'

/* START */
function code() {
	expect(process.env.a ? 1 : 2).toBe(2);
	expect(a ? 1 : 2).toBe(1);
	expect(b ? 1 : 2).toBe(1);
	expect(typeof process.env.b === 'undefined' ? 1 : 2).toBe(1);
	expect(typeof a === 'undefined' ? 1 : 2).toBe(2);
	expect(typeof b === 'undefined' ? 1 : 2).toBe(2);
}
/* END */

it("should work with recursion define", async () => {
	code();
	const content = await fs.promises.readFile(__filename, "utf-8");
	const m = content.match(/\/\* START \*\/([\s\S]*)\/\* END \*\//m);
	expect(m[1].trim()).toBe(code.toString());
});
