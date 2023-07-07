import fs from "fs";
import path from "path";
it("chunks/async-two_js.js should exist", async function () {
	import("./two");

	let chunks = fs.readdirSync(path.resolve(__dirname, "chunks"));

	const chunkFilename = chunks[0];
	const expectedName = "async-two_js";

	expect(chunkFilename.startsWith(expectedName));
	expect(chunkFilename.endsWith(".js"));
	expect(chunkFilename.length).toBe(
		expectedName.length + ".".length + /* chunkhash */ 8 + ".js".length
	);
});
