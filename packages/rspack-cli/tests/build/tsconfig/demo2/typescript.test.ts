import { run } from "../../../utils/test-utils";
import { existsSync } from "fs";
import { resolve } from "path";

describe("tsconfig demo1", () => {
	it("warning tsconfig", async () => {
		const { exitCode, stderr, stdout } = await run(__dirname, []);
		console.log(stderr);
		console.log(stdout);
		expect(stderr).toBeFalsy();
		expect(stdout).toBeTruthy();
		expect(exitCode).toBe(0);
	});

	it("should support specifying config in typescript", async () => {
		const { exitCode, stderr, stdout } = await run(__dirname, [
			"-c",
			"./rspack.config.ts"
		]);

		expect(stderr).toBeFalsy();
		expect(stdout).toBeTruthy();
		expect(exitCode).toBe(0);
		expect(existsSync(resolve(__dirname, "dist/foo.bundle.js"))).toBeTruthy();
	});
});
