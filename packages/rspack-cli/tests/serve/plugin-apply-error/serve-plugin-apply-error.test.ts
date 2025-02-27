import { normalizeStderr, runWatch } from "../../utils/test-utils";

describe("should display plugin error", () => {
	it.concurrent("display error", async () => {
		const { stderr } = await runWatch(__dirname, ["serve"], {
			killString: /Error: /
		});

		expect(normalizeStderr(stderr)).toContain("error in plugin");
	});
});
