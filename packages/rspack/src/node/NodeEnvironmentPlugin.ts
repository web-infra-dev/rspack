/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib/node/NodeEnvironmentPlugin.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */
// @ts-expect-error we directly import from enhanced-resolve inner js file to improve performance
import CachedInputFileSystem from "enhanced-resolve/lib/CachedInputFileSystem";
import fs from "graceful-fs";

import type { Compiler } from "..";
import type { InfrastructureLogging } from "../config";
import {
	type LoggerConsole,
	createConsoleLogger
} from "../logging/createConsoleLogger";
import type { InputFileSystem } from "../util/fs";
import NodeWatchFileSystem from "./NodeWatchFileSystem";
import nodeConsole from "./nodeConsole";

export interface NodeEnvironmentPluginOptions {
	infrastructureLogging: InfrastructureLogging;
}

export default class NodeEnvironmentPlugin {
	options: NodeEnvironmentPluginOptions;

	constructor(options: NodeEnvironmentPluginOptions) {
		this.options = options;
	}

	apply(compiler: Compiler) {
		const { infrastructureLogging } = this.options;
		compiler.infrastructureLogger = createConsoleLogger({
			level: infrastructureLogging.level || "info",
			debug: infrastructureLogging.debug || false,
			console:
				infrastructureLogging.console ||
				(nodeConsole({
					colors: infrastructureLogging.colors,
					appendOnly: infrastructureLogging.appendOnly,
					stream: infrastructureLogging.stream!
				}) as LoggerConsole)
		});

		const inputFileSystem: InputFileSystem = new CachedInputFileSystem(
			fs,
			60000
		);
		compiler.inputFileSystem = inputFileSystem;
		compiler.outputFileSystem = fs;
		compiler.intermediateFileSystem = null;
		compiler.watchFileSystem = new NodeWatchFileSystem(inputFileSystem);
		compiler.hooks.beforeRun.tap("NodeEnvironmentPlugin", compiler => {
			if (compiler.inputFileSystem === inputFileSystem) {
				compiler.fsStartTime = Date.now();
				inputFileSystem.purge?.();
			}
		});
	}
}
