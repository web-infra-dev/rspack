/*
	MIT License http://www.opensource.org/licenses/mit-license.php
	Author Tobias Koppers @sokra
*/

import CachedInputFileSystem from "enhanced-resolve/lib/CachedInputFileSystem";
import fs from "graceful-fs";
import createConsoleLogger from "../logging/createConsoleLogger";
import NodeWatchFileSystem from "./NodeWatchFileSystem";
import nodeConsole from "./nodeConsole";
import type { InfrastructureLogging } from "../config/RspackOptions";
import type { Compiler } from "..";

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
				nodeConsole({
					colors: infrastructureLogging.colors,
					appendOnly: infrastructureLogging.appendOnly,
					stream: infrastructureLogging.stream
				})
		});
		compiler.inputFileSystem = new CachedInputFileSystem(fs, 60000);
		const inputFileSystem = compiler.inputFileSystem;
		compiler.outputFileSystem = fs;
		compiler.intermediateFileSystem = fs;
		compiler.watchFileSystem = new NodeWatchFileSystem(
			compiler.inputFileSystem
		);
		compiler.hooks.beforeRun.tap("NodeEnvironmentPlugin", compiler => {
			if (compiler.inputFileSystem === inputFileSystem) {
				(compiler as any).fsStartTime = Date.now();
				inputFileSystem.purge();
			}
		});
	}
}
