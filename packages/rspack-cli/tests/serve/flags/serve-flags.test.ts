import { normalizeStdout, runWatch } from "../../utils/test-utils";

describe("serve usage with flags", () => {
	it.concurrent("basic flags", async () => {
		const { stdout } = await runWatch(
			__dirname,
			["serve", "--host=localhost", "--port=8888", "--hot"],
			{
				killString: /localhost/
			}
		);

		expect(normalizeStdout(stdout)).toContain(
			'{"hot":true,"host":"localhost","port":8888}'
		);
	});
});
