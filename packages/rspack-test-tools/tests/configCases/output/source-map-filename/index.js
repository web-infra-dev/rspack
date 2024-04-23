import fs from "fs";
import path from "path";
it("source-map-filename/name should same", async function () {
	import("./two");
	expect(
		fs.readdirSync(path.resolve(__dirname, "")).includes("main.js.map")
	).toBe(true);
});
