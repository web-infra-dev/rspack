it("should run with natural chunkIds", async () => {
	await import("./files/file1.js");
	await import("./files/file2.js");
	const distFiles = require("fs").readdirSync(__dirname);
	expect(distFiles).toContain("chunk0.js");
	expect(distFiles).toContain("chunk1.js");
});
