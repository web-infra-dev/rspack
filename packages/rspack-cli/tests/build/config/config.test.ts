import { resolve } from "path";
import { readFile } from "fs/promises";
import { run } from "../../utils/test-utils";

describe("rspack cli", () => {
	describe("should config not found", () => {
		it("should throw an error when config file does not found", async () => {
			const { stderr } = await run(__dirname, ["-c", "not-found-config.js"]);
			expect(stderr).toMatch(/not found/);
		});
	});
	describe("should respect cjs in esm folder", () => {
		const cwd = resolve(__dirname, "./cjs_in_esm");
		it("should load config.cjs file", async () => {
			const { exitCode, stderr, stdout } = await run(cwd, [
				"-c",
				"rspack.config.cjs"
			]);
			expect(stderr).toBeFalsy();
			expect(stdout).toBeTruthy();
			expect(exitCode).toBe(0);
			expect(
				readFile(resolve(cwd, "./dist/cjs.bundle.js"), { encoding: "utf-8" })
			).resolves.toMatch(/Main cjs file/);
		});

		it("should load config.cts file", async () => {
			const { exitCode, stderr, stdout } = await run(cwd, [
				"-c",
				"rspack.config.cts"
			]);
			expect(stderr).toBeFalsy();
			expect(stdout).toBeTruthy();
			expect(exitCode).toBe(0);
			expect(
				readFile(resolve(cwd, "./dist/cts.bundle.js"), { encoding: "utf-8" })
			).resolves.toMatch(/Main cjs file/);
		});
	});
	describe("should load cjs config", () => {
		const cwd = resolve(__dirname, "./cjs");

		it("should load default config.js file", async () => {
			const { exitCode, stderr, stdout } = await run(cwd, []);
			expect(stderr).toBeFalsy();
			expect(stdout).toBeTruthy();
			expect(exitCode).toBe(0);
			expect(
				readFile(resolve(cwd, "./dist/js.bundle.js"), { encoding: "utf-8" })
			).resolves.toMatch(/Main cjs file/);
		});

		it("should load config.ts file", async () => {
			const { exitCode, stderr, stdout } = await run(cwd, [
				"-c",
				"rspack.config.ts"
			]);
			expect(stderr).toBeFalsy();
			expect(stdout).toBeTruthy();
			expect(exitCode).toBe(0);
			expect(
				readFile(resolve(cwd, "./dist/ts.bundle.js"), { encoding: "utf-8" })
			).resolves.toMatch(/Main cjs file/);
		});
		it("should load config.export.ts file", async () => {
			const { exitCode, stderr, stdout } = await run(cwd, [
				"-c",
				"rspack.config.export.ts"
			]);
			expect(stderr).toBeFalsy();
			expect(stdout).toBeTruthy();
			expect(exitCode).toBe(0);
			expect(
				readFile(resolve(cwd, "./dist/ts.bundle.js"), { encoding: "utf-8" })
			).resolves.toMatch(/Main cjs file/);
		});

		it("should load config.cjs file", async () => {
			const { exitCode, stderr, stdout } = await run(cwd, [
				"-c",
				"rspack.config.cjs"
			]);
			expect(stderr).toBeFalsy();
			expect(stdout).toBeTruthy();
			expect(exitCode).toBe(0);
			expect(
				readFile(resolve(cwd, "./dist/cjs.bundle.js"), { encoding: "utf-8" })
			).resolves.toMatch(/Main cjs file/);
		});

		it("should load config.cts file", async () => {
			const { exitCode, stderr, stdout } = await run(cwd, [
				"-c",
				"rspack.config.cts"
			]);
			expect(stderr).toBeFalsy();
			expect(stdout).toBeTruthy();
			expect(exitCode).toBe(0);
			expect(
				readFile(resolve(cwd, "./dist/cts.bundle.js"), { encoding: "utf-8" })
			).resolves.toMatch(/Main cjs file/);
		});
	});

	describe("should load esm config", () => {
		const cwd = resolve(__dirname, "./esm");

		it("should load default config.js file", async () => {
			const { exitCode, stderr, stdout } = await run(cwd, []);
			expect(stderr).toBeFalsy();
			expect(stdout).toBeTruthy();
			expect(exitCode).toBe(0);
			expect(
				readFile(resolve(cwd, "./dist/js.bundle.js"), { encoding: "utf-8" })
			).resolves.toMatch(/Main esm file/);
		});

		it("should load config.ts file", async () => {
			const { exitCode, stdout } = await run(cwd, ["-c", "rspack.config.ts"], {
				nodeOptions: ["--experimental-loader=ts-node/esm"]
			});
			expect(stdout).toBeTruthy();
			expect(exitCode).toBe(0);
			expect(
				readFile(resolve(cwd, "./dist/ts.bundle.js"), { encoding: "utf-8" })
			).resolves.toMatch(/Main esm file/);
		});

		it("should load config.mjs file", async () => {
			const { exitCode, stderr, stdout } = await run(cwd, [
				"-c",
				"rspack.config.mjs"
			]);
			expect(stderr).toBeFalsy();
			expect(stdout).toBeTruthy();
			expect(exitCode).toBe(0);
			expect(
				readFile(resolve(cwd, "./dist/mjs.bundle.js"), { encoding: "utf-8" })
			).resolves.toMatch(/Main esm file/);
		});

		it("should load config.mts file", async () => {
			const { exitCode, stdout } = await run(cwd, ["-c", "rspack.config.mts"], {
				nodeOptions: ["--experimental-loader=ts-node/esm"]
			});

			expect(stdout).toBeTruthy();
			expect(exitCode).toBe(0);
			expect(
				readFile(resolve(cwd, "./dist/mts.bundle.js"), { encoding: "utf-8" })
			).resolves.toMatch(/Main esm file/);
		});
	});

	describe("should load moonrepo config", () => {
		const cwd = resolve(__dirname, "./moonrepo");
		it("should load moonrepo config.ts file", async () => {
			const { exitCode, stdout } = await run(cwd, ["-c", "rspack.config.ts"], {
				nodeOptions: ["--experimental-loader=ts-node/esm"]
			});
			expect(stdout).toBeTruthy();
			expect(exitCode).toBe(0);
			expect(
				readFile(
					resolve(cwd, `./dist/moonrepo.bundle.depsA.1.0.0-depsB.2.0.0.js`),
					{ encoding: "utf-8" }
				)
			).resolves.toMatch(/Main moonrepo file/);
		});
	});
});
