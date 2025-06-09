import fs from "fs";
import { resolve } from "path";
import { run } from "../../utils/test-utils";

const defaultTracePath = "./rspack.pftrace";
const customTracePath = "./custom/trace.json";

function findDefaultOutputDirname() {
	const files = fs.readdirSync(__dirname);
	const file = files.filter(file => file.startsWith(".rspack-profile"));
	return file.length > 0 ? resolve(__dirname, file[0]) : null;
}

describe("profile", () => {
	afterEach(() => {
		const dirname = findDefaultOutputDirname();
		[dirname, resolve(__dirname, customTracePath)].forEach(p => {
			if (p && fs.existsSync(p)) {
				fs.rmSync(p, { recursive: true });
			}
		});
	});

	it("should store all profile files when RSPACK_PROFILE=ALL enabled", async () => {
		const { exitCode } = await run(
			__dirname,
			[],
			{},
			{ RSPACK_PROFILE: "ALL" }
		);
		expect(exitCode).toBe(0);
		const dirname = findDefaultOutputDirname();
		expect(fs.existsSync(resolve(dirname, defaultTracePath))).toBeTruthy();
	});

	it("should store rust trace file when RSPACK_PROFILE=OVERVIEW enabled", async () => {
		const { exitCode } = await run(
			__dirname,
			[],
			{},
			{ RSPACK_PROFILE: "OVERVIEW" }
		);
		expect(exitCode).toBe(0);
		const dirname = findDefaultOutputDirname();
		expect(fs.existsSync(resolve(dirname, defaultTracePath))).toBeTruthy();
	});

	it("should filter trace event when use RSPACK_PROFILE=rspack_resolver,rspack", async () => {
		const { exitCode } = await run(
			__dirname,
			[],
			{},
			{ RSPACK_PROFILE: "rspack,respack_resolver" }
		);
		expect(exitCode).toBe(0);
		const dirname = findDefaultOutputDirname();
		const trace = resolve(dirname, defaultTracePath);
		expect(fs.existsSync(trace)).toBeTruthy();
		const out: { cat?: string }[] = JSON.parse(fs.readFileSync(trace, "utf-8"));
		expect(
			out
				.filter(line => line.cat)
				.every(
					line =>
						line.cat!.startsWith("rspack") ||
						line.cat!.startsWith("disabled-by-default-v8.cpu_profiler")
				)
		).toBe(true);
	});

	it("should be able to customize output path", async () => {
		const { exitCode } = await run(
			__dirname,
			[],
			{},
			{
				RSPACK_PROFILE: "ALL",
				RSPACK_TRACE_OUTPUT: customTracePath
			}
		);
		expect(exitCode).toBe(0);
		expect(fs.existsSync(resolve(__dirname, customTracePath))).toBeTruthy();
	});

	it("should be able to use logger trace layer and default output should be stdout", async () => {
		const { exitCode, stdout } = await run(
			__dirname,
			[],
			{},
			{
				RSPACK_PROFILE: `rspack_core::compiler`,
				RSPACK_TRACE_LAYER: "logger"
			}
		);
		expect(exitCode).toBe(0);
		expect(stdout.includes("rspack_core::compiler")).toBe(true);
	});
});
