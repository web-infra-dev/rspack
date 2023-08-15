import fs from "fs";
import { run } from "../../utils/test-utils";
import { resolve } from "path";

const defaultTracePath = "./rspack.trace";
const defaultJSCPUPath = "./rspack.jscpuprofile";
const customTracePath = "./custom.trace";
const customJSCPUPath = "./custom.jscpuprofile";

describe("profile", () => {
	afterEach(() => {
		[
			defaultTracePath,
			defaultJSCPUPath,
			customTracePath,
			customJSCPUPath
		].forEach(p => {
			const pp = resolve(__dirname, p);
			if (fs.existsSync(pp)) {
				fs.rmSync(pp);
			}
		});
	});

	it("should store rust trace file and js cpu profile file when RSPACK_PROFILE=ALL enabled", async () => {
		const { exitCode } = await run(
			__dirname,
			[],
			{},
			{ RSPACK_PROFILE: "ALL" }
		);
		expect(exitCode).toBe(0);
		expect(fs.existsSync(resolve(__dirname, defaultTracePath))).toBeTruthy();
		expect(fs.existsSync(resolve(__dirname, defaultJSCPUPath))).toBeTruthy();
	});

	it("should store js cpu profile file when RSPACK_PROFILE=JSCPU enabled", async () => {
		const { exitCode } = await run(
			__dirname,
			[],
			{},
			{ RSPACK_PROFILE: "JSCPU" }
		);
		expect(exitCode).toBe(0);
		expect(fs.existsSync(resolve(__dirname, defaultJSCPUPath))).toBeTruthy();
	});

	it("should store rust trace file when RSPACK_PROFILE=TRACE enabled", async () => {
		const { exitCode } = await run(
			__dirname,
			[],
			{},
			{ RSPACK_PROFILE: "TRACE" }
		);
		expect(exitCode).toBe(0);
		expect(fs.existsSync(resolve(__dirname, defaultTracePath))).toBeTruthy();
	});

	it("should filter trace event when use RSPACK_PROFILE=[crate1,crate2]", async () => {
		const { exitCode } = await run(
			__dirname,
			[],
			{},
			{ RSPACK_PROFILE: "[rspack_core]" }
		);
		expect(exitCode).toBe(0);
		const trace = resolve(__dirname, defaultTracePath);
		expect(fs.existsSync(trace)).toBeTruthy();
		const out: { cat?: string }[] = JSON.parse(fs.readFileSync(trace, "utf-8"));
		expect(
			out
				.filter(line => line.cat)
				.every(line => line.cat!.startsWith("rspack_core"))
		).toBe(true);
	});

	it("should be able to customize output path", async () => {
		const { exitCode } = await run(
			__dirname,
			[],
			{},
			{
				RSPACK_PROFILE: `TRACE=output=${customTracePath}|JSCPU=output=${customJSCPUPath}`
			}
		);
		expect(exitCode).toBe(0);
		expect(fs.existsSync(resolve(__dirname, customTracePath))).toBeTruthy();
		expect(fs.existsSync(resolve(__dirname, customJSCPUPath))).toBeTruthy();
	});

	it("should be able to use logger trace layer and default output should be stdout", async () => {
		const { exitCode, stdout } = await run(
			__dirname,
			[],
			{},
			{ RSPACK_PROFILE: `TRACE=layer=logger&filter=rspack_node::plugins` }
		);
		expect(exitCode).toBe(0);
		expect(stdout.includes("rspack_node::plugins")).toBe(true);
	});
});
