import { normalizeStderr, runWatch } from "../../utils/test-utils";

describe("basic serve usage", () => {
	it.only("should work", async () => {
		const { stderr, stdout } = await runWatch(__dirname, ["serve"]);

		expect(normalizeStderr(stderr)).toMatchSnapshot("stderr");
		console.log("stdout:", stdout);
		expect(stdout).toContain("main.js");
	});
});
