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
	it("should work with multiple entries syntax without command (default command)", async () => {
		const { exitCode, stderr, stdout } = await run(__dirname, [
			"./src/index.js",
			"./src/other.js"
		]);

		expect(exitCode).toBe(0);
		expect(stderr).toBeFalsy();
		expect(stdout).toBeTruthy();
	});

	it("should work with multiple entries syntax without command with options (default command)", async () => {
		const { exitCode, stderr, stdout } = await run(__dirname, [
			"./src/index.js",
			"./src/other.js",
			"--mode",
			"development"
		]);

		expect(exitCode).toBe(0);
		expect(stderr).toBeFalsy();
		expect(stdout).toBeTruthy();
	});
	it("should work with multiple entries syntax without command with options #3 (default command)", async () => {
		const { exitCode, stderr, stdout } = await run(__dirname, [
			"./src/index.js",
			"./src/other.js",
			"--entry",
			"./src/again.js"
		]);

		expect(exitCode).toBe(0);
		expect(stderr).toBeFalsy();
		expect(stdout).toBeTruthy();
	});

	it("should work with and override entries from the configuration", async () => {
		const { exitCode, stderr, stdout } = await run(__dirname, [
			"./src/index.js",
			"./src/other.js",
			"--config",
			"./entry.config.js"
		]);
		expect(exitCode).toBe(0);
		expect(stderr).toBeFalsy();
		expect(stdout).toBeTruthy();
	});
	it("should work with configuration return function", async () => {
		const { exitCode, stderr, stdout } = await run(__dirname, [
			"./src/index.js",
			"./src/other.js",
			"--config",
			"./entry.function.js"
		]);
		expect(exitCode).toBe(0);
		expect(stderr).toBeFalsy();
		expect(stdout).toBeTruthy();
	});
	it("should work with configuration return promise", async () => {
		const { exitCode, stderr, stdout } = await run(__dirname, [
			"./src/index.js",
			"./src/other.js",
			"--config",
			"./entry.promise.js"
		]);
		expect(exitCode).toBe(0);
		expect(stderr).toBeFalsy();
		expect(stdout).toBeTruthy();
	});
});
