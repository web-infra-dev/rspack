it("should able to handle filenames with multiple bytes with inline-match-resource", () => {
	expect(require("./你好世界.js!=!./你好世界.js")).toBe("你好世界");
	expect(require("./こんにちは世界.js!=!./こんにちは世界.js")).toBe(
		"こんにちは世界"
	);
	expect(require("./반갑구나세상아.js!=!./반갑구나세상아.js")).toBe(
		"반갑구나,세상아"
	);
});
