import { readFile, run, runWatch } from "../../utils/test-utils";
import { resolve } from "path";

it("should not have `process.env.WEBPACK_SERVE` set on build mode", async () => {
	await run(__dirname, []);
	const mainJs = await readFile(resolve(__dirname, "dist/main.js"), "utf-8");

	expect(mainJs).toContain("WEBPACK_SERVE=<EMPTY>");
});

it("should have `process.env.WEBPACK_SERVE` set on serve mode", async () => {
	await runWatch(__dirname, ["serve"], { killString: /rspack compiled/i });
	const mainJs = await readFile(resolve(__dirname, "dist/main.js"), "utf-8");

	expect(mainJs).toContain("WEBPACK_SERVE=true");
});
