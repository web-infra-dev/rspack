import { RspackOptions } from "@rspack/core";
import { RspackBuildCLIOptions } from "../types";
import { buildConfigWithOptions } from "./buildConfig";

describe("buildConfigWithOptions", () => {
	it("should set mode by NODE_ENV", () => {
		process.env.NODE_ENV = "production";
		const options: RspackBuildCLIOptions = {};
		const item: RspackOptions = { mode: undefined };
		buildConfigWithOptions(item, options, true);
		expect(item.mode).toBe("production");
	});

	it("should set mode by cli options", () => {
		// --mode option on the CLI takes precedence over mode in rspack.config.js
		const options: RspackBuildCLIOptions = { mode: "production" };
		const item: RspackOptions = { mode: "none" };
		buildConfigWithOptions(item, options, true);
		expect(item.mode).toBe("production");
	});

	it("should set watch by cli options", () => {
		const options: RspackBuildCLIOptions = { watch: true };
		const item: RspackOptions = {};
		buildConfigWithOptions(item, options, true);
		expect(item.watch).toBe(true);
	});
});
