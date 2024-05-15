import { readFile, run } from "../../utils/test-utils";
import { resolve } from "path";

it("should not have `process.env.NODE_ENV` when optimization.nodeEnv has been set", async () => {
	await run(__dirname, ["--mode", "production"]);
	const mainJs = await readFile(resolve(__dirname, "dist/main.js"), "utf-8");
	expect(mainJs).toContain("process.env.NODE_ENV");
	expect(mainJs).not.toContain("long_name_should_be_minified");
});
