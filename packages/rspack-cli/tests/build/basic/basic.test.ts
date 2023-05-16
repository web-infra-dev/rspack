import { readFile, run } from "../../utils/test-utils";
import { resolve } from "path";
console.log("version:", process.version);
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
	it("entry option should have higher priority than config", async () => {
		const { exitCode, stderr, stdout } = await run(__dirname, [
			"--entry",
			"./src/other.js",
			"--config",
			"./entry.config.js"
		]);
		const mainJs = await readFile(resolve(__dirname, "dist/main.js"), "utf-8");

		expect(exitCode).toBe(0);
		expect(stderr).toBeFalsy();
		expect(stdout).toBeTruthy();
		expect(mainJs).toContain("other");
		expect(mainJs).not.toContain("CONFIG");
	});
});
