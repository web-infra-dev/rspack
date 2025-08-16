import { run } from "../../utils/test-utils";

describe("plugin apply throw error", () => {
	it.concurrent("should report error", async () => {
		const { stderr } = await run(__dirname);
		console.log(stderr);
		expect(stderr).toMatch(/error in plugin/);
	});
});
