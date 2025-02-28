import {
	getRandomPort,
	normalizeStderr,
	runWatch
} from "../../utils/test-utils";

describe("should run preview command as expected", () => {
	it.concurrent("should work", async () => {
		const port = await getRandomPort();
		const { stderr } = await runWatch(
			__dirname,
			["preview", "--port", port.toString()],
			{
				killString: /localhost/
			}
		);

		expect(normalizeStderr(stderr)).toContain("Project is running at");
	});
});
