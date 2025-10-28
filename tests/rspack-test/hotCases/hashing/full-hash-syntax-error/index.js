import "./module";

const getFile = name =>
	__non_webpack_require__("fs").readFileSync(
		__non_webpack_require__("path").join(__dirname, name),
		"utf-8"
	);

it("should generate the main file and change full hash on update", async () => {
	const hash1 = __webpack_hash__;
	expect(getFile("bundle.js")).toContain(hash1);
	await NEXT_HMR();
	const hash2 = __webpack_hash__;
	expect(hash1).toBeTypeOf("string");
	expect(hash2).toBeTypeOf("string");
	expect(hash2).not.toBe(hash1);
	expect(getFile("bundle.js")).toContain(hash2);
	expect(getFile("bundle.js")).not.toContain(hash1);
	const stats = await NEXT_HMR();
	const hash3 = __webpack_hash__;
	expect(hash1).toBeTypeOf("string");
	expect(hash3).toBeTypeOf("string");
	expect(hash3).not.toBe(hash1);
	expect(getFile("bundle.js")).toContain(hash3);
	expect(getFile("bundle.js")).not.toContain(hash1);
	expect(stats.hash).toBe(hash1);
	await NEXT_HMR();
	const hash4 = __webpack_hash__;
	expect(hash4).toBeTypeOf("string");
	expect(hash4).not.toBe(hash1);
	expect(getFile("bundle.js")).toContain(hash4);
	expect(getFile("bundle.js")).not.toContain(hash1);
	expect(stats.hash).toBe(hash1);
});

import.meta.webpackHot.accept("./module");