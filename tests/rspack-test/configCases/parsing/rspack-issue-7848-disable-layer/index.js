const fs = __non_webpack_require__("node:fs");
const path = __non_webpack_require__("node:path");

const a1 = __webpack_layer__ === "main" ? 'yes' : 'no';
const a2 = __RUNTIME_TYPE__ === "main" ? 'yes' : 'no';
const b1 = __webpack_layer__ === null ? 'yes' : 'no';
const b2 = __RUNTIME_TYPE__ === null ? 'yes' : 'no';
const c1 = typeof __webpack_layer__;
const c2 = typeof __RUNTIME_TYPE__;

it("should work", function () {
	let js = fs.readFileSync(path.resolve(__dirname, "./bundle0.js"), "utf-8");
	js.replace(/\s+/g, " ");

	expect(js.includes("const a1 = false ? 0 : 'no';")).toBeTruthy();
	expect(js.includes("const a2 = false ? 0 : 'no';")).toBeTruthy();
	expect(js.includes("const b1 = true ? 'yes' : 0;")).toBeTruthy();
	expect(js.includes("const b2 = true ? 'yes' : 0;")).toBeTruthy();
	expect(js.includes('const c1 = typeof null;')).toBeTruthy();
	expect(js.includes('const c2 = "object";')).toBeTruthy();

	expect(a1).toBe('no');
	expect(a2).toBe('no');
	expect(b1).toBe('yes');
	expect(b2).toBe('yes');
	expect(c1).toBe("object");
	expect(c2).toBe("object");
});
