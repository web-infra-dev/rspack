import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

it("minimizing an asset file of esm type should success", () => {
	const worker = new URL("./pkg/pkg.js", import.meta.url);
	const minifiedContent = readFileSync(fileURLToPath(worker), "utf-8");
	expect(minifiedContent).toMatchFileSnapshot(path.join(__SNAPSHOT__, 'pkg.js.txt'));
});
