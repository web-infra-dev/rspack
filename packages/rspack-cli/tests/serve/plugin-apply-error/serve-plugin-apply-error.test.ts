import { normalizeStderr, runWatch } from "../../utils/test-utils";

describe("should display plugin error", () => {
	it("display error", async () => {
		const { stderr } = await runWatch(__dirname, ["serve"], {
			killString: /Error: /
		});

		expect(normalizeStderr(stderr)).toContain("error in plugin");
	});
});
