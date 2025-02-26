import { normalizeStderr, runWatch } from "../../utils/test-utils";

describe("should run preview command as expected", () => {
	it("should work", async () => {
		const { stderr } = await runWatch(__dirname, ["preview"], {
			killString: /localhost/
		});

		expect(normalizeStderr(stderr)).toContain("Project is running at");
	});
});
