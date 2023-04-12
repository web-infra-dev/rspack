import { run } from "../../utils/test-utils";
describe("build command", () => {
	it("it should work ", async () => {
		const { exitCode, stderr, stdout } = await run(__dirname, []);
		expect(exitCode).toBe(0);
		expect(stderr).toBeFalsy();
		expect(stdout).toBeTruthy();
	});
	it("should work without command and options (default command)", async () => {
		const { exitCode, stderr, stdout } = await run(__dirname, [
			"--mode",
			"development"
		]);

		expect(exitCode).toBe(0);
		expect(stderr).toBeFalsy();
		expect(stdout).toBeTruthy();
	});
	it("should work with configuration return function", async () => {
		const { exitCode, stderr, stdout } = await run(__dirname, [
			"--config",
			"./entry.function.js"
		]);
		expect(exitCode).toBe(0);
		expect(stderr).toBeFalsy();
		expect(stdout).toBeTruthy();
	});
	it("should work with configuration return promise", async () => {
		const { exitCode, stderr, stdout } = await run(__dirname, [
			"--config",
			"./entry.promise.js"
		]);
		expect(exitCode).toBe(0);
		expect(stderr).toBeFalsy();
		expect(stdout).toBeTruthy();
	});
	it("should work with mjs configuration ", async () => {
		const { exitCode, stderr, stdout } = await run(__dirname, [
			"--config",
			"./entry.config.mjs"
		]);
		expect(exitCode).toBe(0);
		expect(stderr).toBeFalsy();
		expect(stdout).toBeTruthy();
	});
});
