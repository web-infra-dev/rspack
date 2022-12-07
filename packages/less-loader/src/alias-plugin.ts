import { rebaseUrls } from "./url";
import fs from "fs";
interface Options {
	config: {
		resolve: (id: string, dir: string) => string;
	};
	stdinDir: string;
}

export default class LessAliasesPlugin {
	config: Options["config"];
	stdinDir: string;
	constructor(options: Options) {
		this.config = options.config;
		this.stdinDir = options.stdinDir;
	}

	install(less: any, pluginManager: any) {
		const getResolve = (filename: string, currentDirectory: string) => {
			let resolved = this.config.resolve(
				filename,
				currentDirectory || this.stdinDir
			);
			return resolved;
		};
		class AliasPlugin extends less.FileManager {
			config: any;
			constructor(options: Options) {
				super();
				this.config = options.config;
				this.stdinDir = options.stdinDir;
			}

			async loadFile(filename: string, ...args) {
				let resolved;
				let currentDirectory = args[0];
				try {
					// we need to use less's internal loadFile logic to be compatible with less-loader
					return await super.loadFile(filename, ...args);
				} catch (err) {
					resolved = getResolve(filename, currentDirectory);
					const rebasedContents = await rebaseUrls(
						resolved,
						this.stdinDir,
						this.config.resolve
					);
					const contents = rebasedContents.contents
						? rebasedContents.contents
						: fs.readFileSync(resolved, "utf8");
					return {
						filename: resolved,
						contents: contents
					};
				}
			}
		}
		pluginManager.addFileManager(
			new AliasPlugin({ config: this.config, stdinDir: this.stdinDir })
		);
	}
}
