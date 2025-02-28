import { resolve } from "path";
import { readFile } from "fs/promises";
import { run } from "../../utils/test-utils";

describe("rspack cli", () => {
	describe("should config not found", () => {
		it.concurrent(
			"should throw an error when config file does not found",
			async () => {
				const { stderr } = await run(__dirname, ["-c", "not-found-config.js"]);
				expect(stderr).toMatch(/not found/);
			}
		);
	});
	describe("should respect cjs in esm folder", () => {
		const cwd = resolve(__dirname, "./cjs_in_esm");
		it.concurrent("should load config.cjs file", async () => {
			const { exitCode, stderr, stdout } = await run(cwd, [
				"-c",
				"rspack.config.cjs",
				"--output-path",
				"dist/cjs-1"
			]);
			expect(stderr).toBeFalsy();
			expect(stdout).toBeTruthy();
			expect(exitCode).toBe(0);
			expect(
				readFile(resolve(cwd, "./dist/cjs-1/cjs.bundle.js"), {
					encoding: "utf-8"
				})
			).resolves.toMatch(/Main cjs file/);
		});

		it.concurrent("should load config.cts file", async () => {
			const { exitCode, stderr, stdout } = await run(cwd, [
				"-c",
				"rspack.config.cts",
				"--output-path",
				"dist/cts-1"
			]);
			expect(stderr).toBeFalsy();
			expect(stdout).toBeTruthy();
			expect(exitCode).toBe(0);
			expect(
				readFile(resolve(cwd, "./dist/cts-1/cts.bundle.js"), {
					encoding: "utf-8"
				})
			).resolves.toMatch(/Main cjs file/);
		});
	});
	describe("should load cjs config", () => {
		const cwd = resolve(__dirname, "./cjs");

		it.concurrent("should load default config.js file", async () => {
			const { exitCode, stderr, stdout } = await run(cwd, [
				"--output-path",
				"dist/js-1"
			]);
			expect(stderr).toBeFalsy();
			expect(stdout).toBeTruthy();
			expect(exitCode).toBe(0);
			expect(
				readFile(resolve(cwd, "./dist/js-1/js.bundle.js"), {
					encoding: "utf-8"
				})
			).resolves.toMatch(/Main cjs file/);
		});

		it.concurrent("should load config.ts file", async () => {
			const { exitCode, stderr, stdout } = await run(cwd, [
				"-c",
				"rspack.config.ts",
				"--output-path",
				"dist/ts-1"
			]);
			expect(stderr).toBeFalsy();
			expect(stdout).toBeTruthy();
			expect(exitCode).toBe(0);
			expect(
				readFile(resolve(cwd, "./dist/ts-1/ts.bundle.js"), {
					encoding: "utf-8"
				})
			).resolves.toMatch(/Main cjs file/);
		});
		it.concurrent("should load config.export.ts file", async () => {
			const { exitCode, stderr, stdout } = await run(cwd, [
				"-c",
				"rspack.config.export.ts",
				"--output-path",
				"dist/export-1"
			]);
			expect(stderr).toBeFalsy();
			expect(stdout).toBeTruthy();
			expect(exitCode).toBe(0);
			expect(
				readFile(resolve(cwd, "./dist/export-1/ts.bundle.js"), {
					encoding: "utf-8"
				})
			).resolves.toMatch(/Main cjs file/);
		});

		it.concurrent("should load config.cjs file", async () => {
			const { exitCode, stderr, stdout } = await run(cwd, [
				"-c",
				"rspack.config.cjs",
				"--output-path",
				"dist/cjs-2"
			]);
			expect(stderr).toBeFalsy();
			expect(stdout).toBeTruthy();
			expect(exitCode).toBe(0);
			expect(
				readFile(resolve(cwd, "./dist/cjs-2/cjs.bundle.js"), {
					encoding: "utf-8"
				})
			).resolves.toMatch(/Main cjs file/);
		});

		it.concurrent("should load config.cts file", async () => {
			const { exitCode, stderr, stdout } = await run(cwd, [
				"-c",
				"rspack.config.cts",
				"--output-path",
				"dist/cts-2"
			]);
			expect(stderr).toBeFalsy();
			expect(stdout).toBeTruthy();
			expect(exitCode).toBe(0);
			expect(
				readFile(resolve(cwd, "./dist/cts-2/cts.bundle.js"), {
					encoding: "utf-8"
				})
			).resolves.toMatch(/Main cjs file/);
		});
	});

	describe("should load esm config", () => {
		const cwd = resolve(__dirname, "./esm");

		it.concurrent("should load default config.js file", async () => {
			const { exitCode, stderr, stdout } = await run(cwd, [
				"--output-path",
				"dist/js-2"
			]);
			expect(stderr).toBeFalsy();
			expect(stdout).toBeTruthy();
			expect(exitCode).toBe(0);
			expect(
				readFile(resolve(cwd, "./dist/js-2/js.bundle.js"), {
					encoding: "utf-8"
				})
			).resolves.toMatch(/Main esm file/);
		});

		it.concurrent("should load config.ts file", async () => {
			const { exitCode, stdout } = await run(
				cwd,
				["-c", "rspack.config.ts", "--output-path", "dist/ts-2"],
				{
					nodeOptions: ["--experimental-loader=ts-node/esm"]
				}
			);
			expect(stdout).toBeTruthy();
			expect(exitCode).toBe(0);
			expect(
				readFile(resolve(cwd, "./dist/ts-2/ts.bundle.js"), {
					encoding: "utf-8"
				})
			).resolves.toMatch(/Main esm file/);
		});

		it.concurrent("should load config.mjs file", async () => {
			const { exitCode, stderr, stdout } = await run(cwd, [
				"-c",
				"rspack.config.mjs",
				"--output-path",
				"dist/mjs-1"
			]);
			expect(stderr).toBeFalsy();
			expect(stdout).toBeTruthy();
			expect(exitCode).toBe(0);
			expect(
				readFile(resolve(cwd, "./dist/mjs-1/mjs.bundle.js"), {
					encoding: "utf-8"
				})
			).resolves.toMatch(/Main esm file/);
		});

		it.concurrent("should load config.mts file", async () => {
			const { exitCode, stdout } = await run(
				cwd,
				["-c", "rspack.config.mts", "--output-path", "dist/mts-1"],
				{
					nodeOptions: ["--experimental-loader=ts-node/esm"]
				}
			);

			expect(stdout).toBeTruthy();
			expect(exitCode).toBe(0);
			expect(
				readFile(resolve(cwd, "./dist/mts-1/mts.bundle.js"), {
					encoding: "utf-8"
				})
			).resolves.toMatch(/Main esm file/);
		});
	});

	describe("should load config with defineConfig helper", () => {
		const cwd = resolve(__dirname, "./esm");

		it.concurrent("should load config.ts file", async () => {
			const { exitCode, stdout } = await run(
				cwd,
				["-c", "rspack.config.ts", "--output-path", "dist/ts-3"],
				{
					nodeOptions: ["--experimental-loader=ts-node/esm"]
				}
			);
			expect(stdout).toBeTruthy();
			expect(exitCode).toBe(0);
			expect(
				readFile(resolve(cwd, "./dist/ts-3/ts.bundle.js"), {
					encoding: "utf-8"
				})
			).resolves.toMatch(/Main esm file/);
		});

		it.concurrent("should load config.mts file", async () => {
			const { exitCode, stdout } = await run(
				cwd,
				["-c", "rspack.config.mts", "--output-path", "dist/mts-2"],
				{
					nodeOptions: ["--experimental-loader=ts-node/esm"]
				}
			);

			expect(stdout).toBeTruthy();
			expect(exitCode).toBe(0);
			expect(
				readFile(resolve(cwd, "./dist/mts-2/mts.bundle.js"), {
					encoding: "utf-8"
				})
			).resolves.toMatch(/Main esm file/);
		});
	});

	describe("should load monorepo config", () => {
		const cwd = resolve(__dirname, "./monorepo");
		it.concurrent("should load monorepo config.ts file", async () => {
			const { exitCode, stdout } = await run(cwd, ["-c", "rspack.config.ts"], {
				nodeOptions: ["--experimental-loader=ts-node/esm"]
			});
			expect(stdout).toBeTruthy();
			expect(exitCode).toBe(0);
			expect(
				readFile(
					resolve(cwd, `./dist/monorepo.bundle.depsA.1.0.0-depsB.2.0.0.js`),
					{ encoding: "utf-8" }
				)
			).resolves.toMatch(/Main monorepo file/);
		});
	});

	// describe("loose-unrecognized-keys (default)", () => {
	// 	const cwd = resolve(__dirname, "./loose-unrecognized-keys");
	// 	it.concurrent("should report unrecognized keys", async () => {
	// 		const { stderr, exitCode } = await run(cwd, []);
	// 		expect(stderr).toMatchInlineSnapshot(`
	// 		"Configuration error:
	// 		- Unrecognized key(s) in object: '_additionalProperty'"
	// 	`);
	// 		expect(stderr).not.toMatch("ValidationError");
	// 		expect(exitCode).toBe(0);
	// 	});
	// });

	// describe("loose-unrecognized-keys 2 (default)", () => {
	// 	const cwd = resolve(__dirname, "./loose-unrecognized-keys-other-error");
	// 	it.concurrent("should fail on other error", async () => {
	// 		const { stderr, exitCode } = await run(cwd, []);
	// 		expect(stderr).toMatch("ValidationError");
	// 		expect(stderr).toMatch(
	// 			`The provided value "./context" must be an absolute path. at \"context"`
	// 		);
	// 		expect(exitCode).toBe(1);
	// 	});
	// });
});
