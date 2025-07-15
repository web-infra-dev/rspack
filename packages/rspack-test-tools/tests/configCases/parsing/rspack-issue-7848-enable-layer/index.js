const fs = __non_webpack_require__("node:fs");
const path = __non_webpack_require__("node:path");

const a1 = __webpack_layer__ === "main" ? "yes" : "no";
const a2 = __RUNTIME_TYPE__ === "main" ? "yes" : "no";
const b1 = __webpack_layer__ === null ? "yes" : "no";
const b2 = __RUNTIME_TYPE__ === null ? "yes" : "no";
const c1 = typeof __webpack_layer__;
const c2 = typeof __RUNTIME_TYPE__;

it("should work", function () {
	let js = fs.readFileSync(path.resolve(__dirname, "./bundle0.js"), "utf-8");
	js.replace(/\s+/g, " ");

	expect(js.includes("const a1 = true ? 'yes' : 0;")).toBeTruthy();
	expect(js.includes("const a2 = true ? 'yes' : 0;")).toBeTruthy();
	expect(js.includes("const b1 = false ? 0 : 'no';")).toBeTruthy();
	expect(js.includes("const b2 = false ? 0 : 'no';")).toBeTruthy();
	expect(js.includes('const c1 = typeof "main";')).toBeTruthy();
	expect(js.includes('const c2 = "string";')).toBeTruthy();

	expect(a1).toBe("yes");
	expect(a2).toBe("yes");
	expect(b1).toBe("no");
	expect(b2).toBe("no");
	expect(c1).toBe("string");
	expect(c2).toBe("string");
});
