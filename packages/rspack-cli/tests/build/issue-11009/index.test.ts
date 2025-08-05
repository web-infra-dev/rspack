import { resolve } from "path";
import { readFile, run } from "../../utils/test-utils";

it.concurrent(
	"should set config path to persistent cache build dependencies",
	async () => {
		const { stdout } = await run(__dirname, []);
		const mainJs = await readFile(resolve(__dirname, "dist/main.js"), "utf-8");
		expect(mainJs).toContain("entry-issue-11009");
		expect(stdout).toContain("issue-11009/rspack.config.js");
		expect(stdout).toContain("issue-11009/base.config.js");
	}
);
