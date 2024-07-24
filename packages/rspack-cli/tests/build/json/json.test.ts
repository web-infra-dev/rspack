import { existsSync } from "fs";
import { resolve } from "path";
import { readFile, run } from "../../utils/test-utils";

const successMessage = "stats are successfully stored as json to stats.json";

describe("json", () => {
	it("should work and store json to a file", async () => {
		const { exitCode, stderr, stdout } = await run(__dirname, [
			"--json",
			"stats.json"
		]);
		expect(exitCode).toBe(0);
		expect(stderr).toContain(successMessage);
		expect(stdout).toBeFalsy();
		expect(existsSync(resolve(__dirname, "./stats.json"))).toBeTruthy();

		let data;

		try {
			data = await readFile(resolve(__dirname, "stats.json"), "utf-8");
		} catch (error) {
			expect(error).toBe(null);
		}
		expect(JSON.parse(data)["time"]).toBeTruthy();
		expect(() => JSON.parse(data)).not.toThrow();
	});
});
