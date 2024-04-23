import fs from "fs";
import path from "path";
it("should generate async chunk", async function () {
	import("./two");
	let chunks = fs.readdirSync(path.resolve(__dirname, "chunks"));
	expect(/async-[0-9]*\.[0-9a-z]{8}\.js/.test(chunks[0])).toBeTruthy();
});
