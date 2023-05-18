import { run } from "../../utils/test-utils";
import { existsSync, writeFileSync, unlinkSync } from "fs";
import { resolve } from "path";

describe("rspack cli", () => {
	describe("should config not found", () => {
		it("should throw an error when config file does not found", async () => {
			const { stderr } = await run(__dirname, ["-c", "not-found-config.js"]);
			expect(stderr).toMatch(/not found/);
		});
	});

	describe("should load cjs config", () => {
		const cwd = resolve(__dirname, "./cjs");

		it("should load default config.js file", async () => {
			const { exitCode, stderr, stdout } = await run(cwd, []);
			expect(stderr).toBeFalsy();
			expect(stdout).toBeTruthy();
			expect(exitCode).toBe(0);
			expect(existsSync(resolve(cwd, "./dist/js.bundle.js"))).toBeTruthy();
		});

		it("should load config.ts file", async () => {
			const { exitCode, stderr, stdout } = await run(cwd, [
				"-c",
				"rspack.config.ts"
			]);
			expect(stderr).toBeFalsy();
			expect(stdout).toBeTruthy();
			expect(exitCode).toBe(0);
			expect(existsSync(resolve(cwd, "./dist/ts.bundle.js"))).toBeTruthy();
		});

		it("should load config.cjs file", async () => {
			const { exitCode, stderr, stdout } = await run(cwd, [
				"-c",
				"rspack.config.cjs"
			]);
			expect(stderr).toBeFalsy();
			expect(stdout).toBeTruthy();
			expect(exitCode).toBe(0);
			expect(existsSync(resolve(cwd, "./dist/cjs.bundle.js"))).toBeTruthy();
		});

		it("should load config.cts file", async () => {
			const { exitCode, stderr, stdout } = await run(cwd, [
				"-c",
				"rspack.config.cts"
			]);
			expect(stderr).toBeFalsy();
			expect(stdout).toBeTruthy();
			expect(exitCode).toBe(0);
			expect(existsSync(resolve(cwd, "./dist/cts.bundle.js"))).toBeTruthy();
		});
	});

	describe("should load esm config", () => {
		const cwd = resolve(__dirname, "./esm");

		const packageJsonPath = resolve(cwd, "./package.json");
		beforeAll(() => {
			// mock user's package.json type is module
			writeFileSync(packageJsonPath, `{"type": "module"}`);
		});
		afterAll(() => unlinkSync(packageJsonPath));

		it("should load default config.js file", async () => {
			const { exitCode, stderr, stdout } = await run(cwd, []);
			expect(stderr).toBeFalsy();
			expect(stdout).toBeTruthy();
			expect(exitCode).toBe(0);
			expect(existsSync(resolve(cwd, "./dist/js.bundle.js"))).toBeTruthy();
		});

		it("should load config.ts file", async () => {
			const { exitCode, stderr, stdout } = await run(cwd, [
				"-c",
				"rspack.config.ts"
			]);
			expect(stderr).toBeFalsy();
			expect(stdout).toBeTruthy();
			expect(exitCode).toBe(0);
			expect(existsSync(resolve(cwd, "./dist/ts.bundle.js"))).toBeTruthy();
		});

		it("should load config.mjs file", async () => {
			const { exitCode, stderr, stdout } = await run(cwd, [
				"-c",
				"rspack.config.mjs"
			]);
			expect(stderr).toBeFalsy();
			expect(stdout).toBeTruthy();
			expect(exitCode).toBe(0);
			expect(existsSync(resolve(cwd, "./dist/mjs.bundle.js"))).toBeTruthy();
		});

		it("should load config.mts file", async () => {
			const { exitCode, stderr, stdout } = await run(cwd, [
				"-c",
				"rspack.config.mts"
			]);
			expect(stderr).toBeFalsy();
			expect(stdout).toBeTruthy();
			expect(exitCode).toBe(0);
			expect(existsSync(resolve(cwd, "./dist/mts.bundle.js"))).toBeTruthy();
		});
	});
});
