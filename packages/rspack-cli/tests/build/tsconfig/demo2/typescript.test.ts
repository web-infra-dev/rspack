import { run } from "../../../utils/test-utils";
import { existsSync } from "fs";
import { resolve } from "path";

describe("tsconfig demo1", () => {
	it("warning tsconfig", async () => {
		const { exitCode, stderr, stdout } = await run(__dirname, []);
		expect(stderr).toBeFalsy();
		expect(stdout).toBeTruthy();
		expect(exitCode).toBe(0);
	});
});
