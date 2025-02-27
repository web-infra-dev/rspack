import { resolve } from "path";
import { readFile, run, runWatch } from "../../utils/test-utils";

it.concurrent(
	"should not have `process.env.WEBPACK_SERVE` set on build mode",
	async () => {
		await run(__dirname, []);
		const mainJs = await readFile(resolve(__dirname, "dist/main.js"), "utf-8");

		expect(mainJs).toContain("WEBPACK_SERVE=<EMPTY>");
	}
);

// TODO: this test is not stable on CI, need to fix it
it.skip("should have `process.env.WEBPACK_SERVE` set on serve mode", async () => {
	await runWatch(__dirname, ["serve"], { killString: /rspack compiled/i });
	// wait 1s to make sure the assets have been generated
	await new Promise(resolve => setTimeout(resolve, 1000));
	const mainJs = await readFile(resolve(__dirname, "dist/main.js"), "utf-8");

	expect(mainJs).toContain("WEBPACK_SERVE=true");
});
