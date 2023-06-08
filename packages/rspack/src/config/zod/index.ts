import { z } from "zod";
import { entry } from "./entry";
import { experiments } from "./experiments";
import { externals } from "./externals";
import { externalsType } from "./externals-type";
import { externalsPresets } from "./externals-presets";
import { infrastructureLogging } from "./infrastructure-logging";
import { node } from "./node";
import { builtins } from "./builtins";
import { watchOptions } from "./watch-options";
import { target } from "./target";
import { stats } from "./stats";
import { snapshot } from "./snapshot";
import { output } from "./output";
import { devtool } from "./devtool";
import { optimization } from "./optimization";
import { resolve } from "./resolve";

export function configSchema() {
	return z
		.object({
			target: target().optional(),
			mode: z.enum(["development", "production", "none"]).optional(),
			entry: entry().optional(),
			experiments: experiments().optional(),
			externals: externals().optional(),
			externalsType: externalsType().optional(),
			externalsPresets: externalsPresets().optional(),
			infrastructureLogging: infrastructureLogging().optional(),
			cache: z.boolean().optional(),
			context: z.string().optional(),
			dependencies: z.string().array().optional(),
			devtool: devtool().optional(),
			node: node().optional(),
			ignoreWarnings: z.instanceof(RegExp).or(z.function()).array().optional(),
			watchOptions: watchOptions().optional(),
			watch: z.boolean().optional(),
			stats: stats().optional(),
			snapshot: snapshot().optional(),
			optimization: optimization().optional(),
			resolve: resolve().optional(),
			// TODO(hyf0): what's the usage of this?
			name: z.string().optional(),
			// TODO
			devServer: z.object({}).optional(),
			output: output().optional(),
			plugins: z.any().optional(),
			builtins: builtins().optional(),
			module: z.any().optional()
		})
		.strict();
}
