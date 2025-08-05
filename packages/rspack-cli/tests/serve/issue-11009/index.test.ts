import { runWatch } from "../../utils/test-utils";

describe("should set config path to persistent cache build dependencies", () => {
	it.concurrent("should serve work", async () => {
		const { stdout } = await runWatch(__dirname, ["serve"], {
			killString: /localhost/
		});

		expect(stdout).toContain("issue-11009/rspack.config.js");
		expect(stdout).toContain("issue-11009/base.config.js");
	});
});
