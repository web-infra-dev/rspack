import path from "node:path";
import type { Compiler } from "@rspack/core";
import fs from "fs-extra";
import { rimrafSync } from "rimraf";

async function loopFile(
	dir: string,
	callback: (filePath: string, content: string) => void
) {
	const children = await fs.readdir(dir);
	await Promise.all(
		children.map(async filename => {
			const filePath = path.join(dir, filename);
			const stat = await fs.stat(filePath);
			if (stat.isFile()) {
				const content = await fs.readFile(filePath);
				callback(filePath, content.toString());
			} else if (stat.isDirectory()) {
				return loopFile(filePath, callback);
			}
		})
	);
}

const PLUGIN_NAME = "HotUpdatePlugin";

export class HotUpdatePlugin {
	private initialized = false;
	private updateIndex = 0;
	private files: Record<string, string[]> = {};
	constructor(
		private projectDir: string,
		private tempDir: string
	) {}

	private getContent(filePath: string, index: number) {
		const contents = this.files[filePath] || [];
		let content =
			contents[index] === undefined ? contents.at(-1) || "" : contents[index];
		let command = "";
		const matchResult = content.match(/^<(.+?)>([\w\W]*)$/);
		if (matchResult) {
			command = matchResult[1];
			content = matchResult[2];
		}
		return {
			content,
			command
		};
	}
	private async updateFiles() {
		await Promise.all(
			this.getModifiedFiles().map(async filePath => {
				const { content, command } = this.getContent(
					filePath,
					this.updateIndex
				);
				// match command
				if (command === "delete") {
					await fs.remove(filePath);
					return;
				}
				if (command === "force_write") {
					await fs.writeFile(filePath, content);
					return;
				}
				// default
				const { content: oldContent } = this.getContent(
					filePath,
					this.updateIndex - 1
				);
				if (this.updateIndex !== 0 && content === oldContent) {
					return;
				}
				await fs.writeFile(filePath, content);
			})
		);
	}

	async initialize() {
		if (this.initialized) {
			return;
		}
		this.initialized = true;
		rimrafSync(this.tempDir);
		fs.copySync(this.projectDir, this.tempDir);

		await loopFile(this.tempDir, (filePath, content) => {
			const contents = content.split(/---+\r?\n/g);
			if (contents.length > 1) {
				this.files[filePath] = contents;
			}
		});
		await this.updateFiles();
	}

	getModifiedFiles() {
		return Object.keys(this.files);
	}

	getUpdateIndex() {
		return this.updateIndex;
	}
	getTotalUpdates() {
		return Object.values(this.files).reduce((max, item) => {
			return Math.max(max, item.length);
		}, 1);
	}
	async goNext() {
		this.updateIndex++;
		await this.updateFiles();
	}

	apply(compiler: Compiler) {
		const options = compiler.options;
		options.context = this.tempDir;
		options.module.rules ??= [];
		options.module.rules.push({
			test: /\.(js|css|json)/,
			use: [
				{
					loader: path.resolve(__dirname, "./loader.js")
				}
			]
		});
		let isRebuild = false;
		compiler.hooks.beforeRun.tap(PLUGIN_NAME, () => {
			compiler.modifiedFiles = new Set(
				isRebuild ? this.getModifiedFiles() : []
			);
			isRebuild = true;
		});

		compiler.hooks.compilation.tap(PLUGIN_NAME, compilation => {
			compilation.hooks.additionalTreeRuntimeRequirements.tap(
				PLUGIN_NAME,
				(_chunk: any, set: any) => {
					set.add(compiler.webpack.RuntimeGlobals.moduleCache);
				}
			);
			compilation.hooks.runtimeModule.tap(
				PLUGIN_NAME,
				(module: any, _set: any) => {
					if (module.constructorName === "DefinePropertyGettersRuntimeModule") {
						module.source.source = Buffer.from(
							`
										__webpack_require__.d = function (exports, definition) {
												for (var key in definition) {
														if (__webpack_require__.o(definition, key) && !__webpack_require__.o(exports, key)) {
																Object.defineProperty(exports, key, { configurable: true, enumerable: true, get: definition[key] });
														}
												}
										};
										`,
							"utf-8"
						);
					}
				}
			);
		});
	}
}
