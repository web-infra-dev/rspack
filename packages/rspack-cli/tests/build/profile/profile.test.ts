import fs from "fs";
import { run } from "../../utils/test-utils";

import { resolve } from "path";

describe("profile", () => {
	afterEach(() => {
		const jscpuprofilePath = resolve(__dirname, "./rspack.jscpuprofile");
		if (fs.existsSync(jscpuprofilePath)) {
			fs.rmSync(jscpuprofilePath);
		}
		const rustTracePath = resolve(__dirname, "./rspack.trace");
		if (fs.existsSync(rustTracePath)) {
			fs.rmSync(rustTracePath);
		}
	});

	it("should store js cpu profile file when RSPACK_PROFILE=JSCPU enabled", async () => {
		const { exitCode } = await run(
			__dirname,
			[],
			{},
			{ RSPACK_PROFILE: "JSCPU" }
		);
		expect(exitCode).toBe(0);
		expect(
			fs.existsSync(resolve(__dirname, "./rspack.jscpuprofile"))
		).toBeTruthy();
	});

	it("should store rust trace file when RSPACK_PROFILE=TRACE enabled", async () => {
		const { exitCode } = await run(
			__dirname,
			[],
			{},
			{ RSPACK_PROFILE: "TRACE" }
		);
		expect(exitCode).toBe(0);
		expect(fs.existsSync(resolve(__dirname, "./rspack.trace"))).toBeTruthy();
	});

	it("should store rust trace file and js cpu profile file when RSPACK_PROFILE=ALL enabled", async () => {
		const { exitCode } = await run(
			__dirname,
			[],
			{},
			{ RSPACK_PROFILE: "ALL" }
		);
		expect(exitCode).toBe(0);
		expect(fs.existsSync(resolve(__dirname, "./rspack.trace"))).toBeTruthy();
		expect(
			fs.existsSync(resolve(__dirname, "./rspack.jscpuprofile"))
		).toBeTruthy();
	});
});
