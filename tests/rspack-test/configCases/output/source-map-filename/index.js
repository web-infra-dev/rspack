import fs from "fs/promises";
import path from "path";

it("source-map-filename/name should same", async function () {
	import("./two");

	expect(async () => await fs.stat(path.resolve(__dirname, "../maps/main.js.map"))).not.toThrow();

	const outputCode = await fs.readFile(__filename, 'utf-8');
	const sourceMapPath = outputCode.match(/\/\/# sourceMappingURL=(.*)/)?.[1];
	expect(sourceMapPath).toBe(path.normalize("maps/main.js.map"));
});
