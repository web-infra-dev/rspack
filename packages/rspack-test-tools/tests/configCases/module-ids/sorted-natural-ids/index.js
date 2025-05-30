import './first';
import './second';
import './third';
import './fourth';
it("sorted natural ids", async () => {
	const fs = __non_webpack_require__("fs");
	const path = __non_webpack_require__("path");
	const source = fs.readFileSync(path.join(__dirname, "bundle0.js"), "utf-8");
	const matched = [...source.matchAll(/(\d) module/g)].map(m => m[1]);
	expect(matched).toEqual(['1', '2', '3', '4'])
})
