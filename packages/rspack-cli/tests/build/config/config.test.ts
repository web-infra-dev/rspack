import { run } from "../../utils/test-utils";
import { writeFileSync, mkdirSync, unlinkSync } from "fs";
import { rm, readFile } from "fs/promises";
import { join, resolve } from "path";

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

		const packageJsonPath = resolve(cwd, "./package.json");
		const packageADepsNodeModules = resolve(
			cwd,
			"./packageA/node_modules/test-deps"
		);
		const packageBDepsNodeModules = resolve(
			cwd,
			"./packageB/node_modules/test-deps"
		);

		beforeAll(() => {
			// mock user's package.json type is module
			writeFileSync(packageJsonPath, `{"type": "module"}`);
			// mock packageA
			mkdirSync(packageADepsNodeModules, { recursive: true });
			writeFileSync(
				join(packageADepsNodeModules, "./package.json"),
				`{ "version": "1.0.0", "name" : "test-deps", "main" : "./index.mjs" }`
			);
			writeFileSync(
				join(packageADepsNodeModules, "./index.mjs"),
				`
				import { readFileSync } from "node:fs";
        import { join } from "node:path";
				export default JSON.parse(readFileSync(join("${packageADepsNodeModules}", "./package.json")));
				`
			);

			// mock packageB
			mkdirSync(packageBDepsNodeModules, { recursive: true });
			writeFileSync(
				join(packageBDepsNodeModules, "./package.json"),
				`{ "version": "2.0.0", "name" : "test-deps", "main" : "./index.mjs" }`
			);
			writeFileSync(
				join(packageBDepsNodeModules, "./index.mjs"),
				`
				import { readFileSync } from "node:fs";
        import { join } from "node:path";
				export default JSON.parse(readFileSync(join("${packageBDepsNodeModules}", "./package.json")));
				`
			);
		});

		afterAll(() => {
			unlinkSync(packageJsonPath);
			rm(packageADepsNodeModules, { recursive: true, force: true });
			rm(packageBDepsNodeModules, { recursive: true, force: true });
		});

		it("should load moonrepo config.ts file", async () => {
			const { exitCode, stderr, stdout } = await run(
				cwd,
				["-c", "rspack.config.ts"],
				{
					nodeOptions: ["--experimental-loader=ts-node/esm"]
				}
			);
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
