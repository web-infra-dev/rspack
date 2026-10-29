import fs from "fs";
import path from "path";

it("should reuse the merged chunk when a cache group has the name of a merged chunk", async () => {
	const named = await import(/* webpackChunkName: "named" */ "./lazy");
	const unnamed = await import("./lazy");
	expect(named.default).toBe("lazy");
	expect(unnamed).toBe(named);
	expect(fs.existsSync(path.resolve(__dirname, "./named.js"))).toBe(true);
});
