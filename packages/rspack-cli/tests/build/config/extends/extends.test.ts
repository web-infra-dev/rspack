import { resolve } from "path";
import { readFile } from "fs/promises";
import { run } from "../../../utils/test-utils";

describe("rspack extends feature", () => {
	describe("basic extends", () => {
		const cwd = resolve(__dirname, "./base");

		it.concurrent("should extend from base config", async () => {
			const { exitCode, stderr, stdout } = await run(cwd, []);

			expect(stderr).toBeFalsy();
			expect(stdout).toBeTruthy();
			expect(exitCode).toBe(0);

			// Check if the output file has the correct name (from base config)
			const outputContent = await readFile(
				resolve(cwd, "./dist/base.bundle.js"),
				{
					encoding: "utf-8"
				}
			);

			expect(outputContent).toMatch(/Base extends test/);
		});
	});

	describe("nested extends", () => {
		const cwd = resolve(__dirname, "./nested");

		it.concurrent("should extend from nested configs", async () => {
			const { exitCode, stderr, stdout } = await run(cwd, []);

			expect(stderr).toBeFalsy();
			expect(stdout).toBeTruthy();
			expect(exitCode).toBe(0);

			// Check if the output file has the correct name (from base config)
			const outputContent = await readFile(
				resolve(cwd, "./dist/base.bundle.js"),
				{
					encoding: "utf-8"
				}
			);

			expect(outputContent).toMatch(/Nested extends test/);
		});
	});

	describe("multiple extends", () => {
		const cwd = resolve(__dirname, "./multiple");

		it.concurrent("should extend from multiple configs", async () => {
			const { exitCode, stderr, stdout } = await run(cwd, []);

			expect(stderr).toBeFalsy();
			expect(stdout).toBeTruthy();
			expect(exitCode).toBe(0);

			// Check if the output file has the correct name (from base config)
			const outputContent = await readFile(
				resolve(cwd, "./dist/base.bundle.js"),
				{
					encoding: "utf-8"
				}
			);

			expect(outputContent).toMatch(/Multiple extends test/);

			// Check if the devtool from dev config is applied
			expect(outputContent).toMatch(/eval-source-map/);
		});
	});

	describe("function config with extends", () => {
		const cwd = resolve(__dirname, "./function");

		it.concurrent("should handle extends in function configs", async () => {
			const { exitCode, stderr, stdout } = await run(cwd, []);

			expect(stderr).toBeFalsy();
			expect(stdout).toBeTruthy();
			expect(exitCode).toBe(0);

			// Check if the output file has the correct name (from base config)
			const outputContent = await readFile(
				resolve(cwd, "./dist/base.bundle.js"),
				{
					encoding: "utf-8"
				}
			);

			expect(outputContent).toMatch(/Function extends test/);
		});
	});

	describe("node module extends", () => {
		const cwd = resolve(__dirname, "./node-module");

		it.concurrent("should extend from a node module", async () => {
			const { exitCode, stderr, stdout } = await run(cwd, []);

			expect(stderr).toBeFalsy();
			expect(stdout).toBeTruthy();
			expect(exitCode).toBe(0);

			// Check if the output file has the correct name (from node module config)
			const outputContent = await readFile(
				resolve(cwd, "./dist/node-module.bundle.js"),
				{
					encoding: "utf-8"
				}
			);

			expect(outputContent).toMatch(/Node module extends test/);
		});
	});

	describe("backward compatibility", () => {
		// Use an existing test directory that doesn't use the extends feature
		const cwd = resolve(__dirname, "../../basic");

		it.concurrent("should not break existing functionality", async () => {
			const { exitCode, stderr, stdout } = await run(cwd, [
				"--config",
				"./entry.config.js"
			]);

			expect(stderr).toBeFalsy();
			expect(stdout).toBeTruthy();
			expect(exitCode).toBe(0);

			// Check if the output file is generated correctly
			const outputExists = await readFile(resolve(cwd, "./dist/main.js"), {
				encoding: "utf-8"
			})
				.then(() => true)
				.catch(() => false);

			expect(outputExists).toBe(true);
		});
	});

	describe("error handling", () => {
		const cwd = resolve(__dirname);

		it.concurrent(
			"should throw an error when extended config file is not found",
			async () => {
				const { exitCode, stderr } = await run(cwd, [
					"-c",
					"./error/not-found.config.js"
				]);

				expect(exitCode).not.toBe(0);
				expect(stderr).toMatch(/not found/);
			}
		);
	});
});
