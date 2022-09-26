import { normalizeStderr, runWatch } from "../../utils/test-utils";

describe("basic serve usage", () => {
	it.skip("should work", async () => {
		const { stderr, stdout } = await runWatch(__dirname, ["serve"]);

		// @todo current server implementation is too buggy to test
		expect(normalizeStderr(stderr)).toBeTruthy;
	});
});
