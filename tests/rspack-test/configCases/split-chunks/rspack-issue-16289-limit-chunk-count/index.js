import fs from "fs";
import path from "path";

it("should reuse the merged chunk when a cache group has the name of a merged chunk", async () => {
	const a = await import("./a");
	const b = await import(/* webpackChunkName: "named" */ "./b");
	expect(a.default).toBe("a");
	expect(b.default).toBe("b");
	expect(fs.existsSync(path.resolve(__dirname, "./named.js"))).toBe(true);
});
