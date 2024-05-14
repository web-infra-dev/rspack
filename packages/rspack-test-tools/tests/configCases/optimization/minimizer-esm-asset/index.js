import { fileURLToPath } from "node:url";
import { readFileSync } from "node:fs";

it("minimizing an asset file of esm type should success", () => {
	const worker = new URL("./pkg/pkg.js", import.meta.url);
	const minifiedContent = readFileSync(fileURLToPath(worker), "utf-8");
	expect(minifiedContent).toMatchSnapshot();
});
