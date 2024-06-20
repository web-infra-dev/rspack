import path from "path";
import { Chunk } from "@rspack/core";
import fs from "fs-extra";

import { escapeEOL, escapeSep, replacePaths } from "../helper";
import { THotStepRuntimeData } from "../runner";
import {
	ECompilerType,
	ITestContext,
	ITestEnv,
	TCompilerOptions,
	TCompilerStats,
	TCompilerStatsCompilation,
	TUpdateOptions
} from "../type";
import { HotProcessor, IHotProcessorOptions } from "./hot";

const NOOP_SET = new Set();

const escapeLocalName = (str: string) => str.split(/[-<>:"/|?*.]/).join("_");

type TModuleGetHandler = (
	file: string,
	options: TCompilerOptions<ECompilerType>
) => string[];

declare var global: {
	self?: {
		[key: string]: (name: string, modules: Record<string, unknown>) => void;
	};
};

const SELF_HANDLER = (
	file: string,
	options: TCompilerOptions<ECompilerType>
): string[] => {
	let res: string[] = [];
	const hotUpdateGlobal = (_: string, modules: Record<string, unknown>) => {
		res = Object.keys(modules);
	};
	const hotUpdateGlobalKey = escapeLocalName(
		`${options.output?.hotUpdateGlobal || "webpackHotUpdate"}${
			options.output?.uniqueName || ""
		}`
	);
	global["self"] ??= {};
	global["self"][hotUpdateGlobalKey] = hotUpdateGlobal;
	require(file);
	delete global["self"][hotUpdateGlobalKey];
	if (!Object.keys(global["self"]).length) {
		delete global["self"];
	}
	return res;
};

const NODE_HANDLER = (file: string): string[] => {
	return Object.keys(require(file).modules) || [];
};

const GET_MODULE_HANDLER: Record<string, TModuleGetHandler> = {
	web: SELF_HANDLER,
	webworker: SELF_HANDLER,
	"async-node": NODE_HANDLER,
	node: NODE_HANDLER
};

type TSupportTarget = keyof typeof GET_MODULE_HANDLER;

export interface IHotSnapshotProcessorOptions<T extends ECompilerType>
	extends IHotProcessorOptions<T> {
	getModuleHandler?: TModuleGetHandler;
	snapshot?: string;
}

export class HotSnapshotProcessor<
	T extends ECompilerType
> extends HotProcessor<T> {
	private hashes: string[] = [];
	private entries: Record<string, string[]> = {};

	constructor(protected _hotOptions: IHotSnapshotProcessorOptions<T>) {
		super(_hotOptions);
	}

	async run(env: ITestEnv, context: ITestContext) {
		context.setValue(
			this._options.name,
			"hotUpdateStepChecker",
			(
				hotUpdateContext: TUpdateOptions,
				stats: TCompilerStats<T>,
				runtime: THotStepRuntimeData
			) => {
				const statsJson: TCompilerStatsCompilation<T> = stats.toJson({
					assets: true,
					chunks: true
				});
				// @ts-expect-error: Some chunk fields are missing from rspack
				let chunks = Array.from(stats?.compilation.chunks || NOOP_SET);
				for (let entry of chunks.filter(i => i.hasRuntime())) {
					if (!this.entries[entry.id!]) {
						this.entries[entry.id!] = Array.from(entry.runtime);
					}
				}
				this.matchStepSnapshot(
					env,
					context,
					hotUpdateContext.updateIndex,
					statsJson,
					runtime
				);
				this.hashes.push(stats.hash!);
			}
		);
		context.setValue(
			this._options.name,
			"hotUpdateStepErrorChecker",
			(
				_: TUpdateOptions,
				stats: TCompilerStats<T>,
				runtime: THotStepRuntimeData
			) => {
				this.hashes.push(stats.hash!);
			}
		);
		await super.run(env, context);
	}

	async check(env: ITestEnv, context: ITestContext) {
		const compiler = this.getCompiler(context);
		const stats = compiler.getStats();
		if (!stats || !stats.hash) {
			env.expect(false);
			return;
		}
		const statsJson = stats.toJson({ assets: true, chunks: true });
		// @ts-expect-error: Some chunk fields are missing from rspack
		let chunks = Array.from(stats?.compilation.chunks || NOOP_SET);
		for (let entry of chunks.filter(i => i.hasRuntime())) {
			this.entries[entry.id!] = Array.from(entry.runtime);
		}
		let matchFailed: Error | null = null;
		try {
			this.matchStepSnapshot(env, context, 0, statsJson);
		} catch (e) {
			matchFailed = e as Error;
		}
		this.hashes.push(stats.hash!);
		if (matchFailed) {
			throw matchFailed;
		}
	}

	protected matchStepSnapshot(
		env: ITestEnv,
		context: ITestContext,
		step: number,
		stats: TCompilerStatsCompilation<T>,
		runtime?: THotStepRuntimeData
	) {
		const compiler = this.getCompiler(context);
		const compilerOptions = compiler.getOptions();
		const getModuleHandler =
			this._hotOptions.getModuleHandler ||
			GET_MODULE_HANDLER[compilerOptions.target as TSupportTarget];
		env.expect(typeof getModuleHandler).toBe("function");

		const lastHash = this.hashes[this.hashes.length - 1];
		const snapshotPath = context.getSource(
			`${this._hotOptions.snapshot || `__snapshots__/${compilerOptions.target}`}/${step}.snap.txt`
		);
		const title = `Case ${path.basename(this._options.name)}: Step ${step}`;
		const hotUpdateFile: Array<{
			name: string;
			content: string;
			modules: string[];
			runtime: string[];
		}> = [];
		const hotUpdateManifest: Array<{ name: string; content: string }> = [];
		const changedFiles: string[] = this.updateOptions.changedFiles.map(
			(i: string) => escapeSep(path.relative(context.getSource(), i))
		);
		changedFiles.sort();

		const hashes: Record<string, string> = {
			[lastHash || "LAST_HASH"]: "LAST_HASH",
			[stats.hash!]: "CURRENT_HASH"
		};

		// TODO: find a better way
		// replace [runtime] to [runtime of id] to prevent worker hash
		const runtimes: Record<string, string> = {};
		for (let [id, runtime] of Object.entries(this.entries)) {
			if (typeof runtime === "string") {
				if (runtime !== id) {
					runtimes[runtime] = `[runtime of ${id}]`;
				}
			} else if (Array.isArray(runtime)) {
				for (let r of runtime) {
					if (r !== id) {
						runtimes[r] = `[runtime of ${id}]`;
					}
				}
			}
		}

		const replaceContent = (str: string) => {
			for (let [raw, replacement] of Object.entries(hashes)) {
				str = str.split(raw).join(replacement);
			}
			// handle timestamp in css-extract
			str = str.replace(/\/\/ (\d+)\s+(?=var cssReload)/, "");
			return replacePaths(str);
		};

		const replaceFileName = (str: string) => {
			for (let [raw, replacement] of Object.entries({
				...hashes,
				...runtimes
			})) {
				str = str.split(raw).join(replacement);
			}
			return str;
		};

		const fileList = stats
			.assets!.map(i => {
				const fileName = i.name;
				const renderName = replaceFileName(fileName);
				const content = replaceContent(
					fs.readFileSync(context.getDist(fileName), "utf-8")
				);
				if (fileName.endsWith("hot-update.js")) {
					const modules = getModuleHandler(
						context.getDist(fileName),
						compilerOptions
					);
					const runtime: string[] = [];
					for (let i of content.matchAll(
						/\/\/ (webpack\/runtime\/[\w_-]+)\s*\n/g
					)) {
						runtime.push(i[1]);
					}
					modules.sort();
					runtime.sort();
					hotUpdateFile.push({
						name: renderName,
						content,
						modules,
						runtime
					});
					return `- Update: ${renderName}, size: ${content.length}`;
				} else if (fileName.endsWith("hot-update.json")) {
					const manifest = JSON.parse(content);
					manifest.c?.sort();
					manifest.r?.sort();
					manifest.m?.sort();
					hotUpdateManifest.push({
						name: renderName,
						content: JSON.stringify(manifest)
					});
					return `- Manifest: ${renderName}, size: ${i.size}`;
				} else if (fileName.endsWith(".js")) {
					return `- Bundle: ${renderName}`;
				}
			})
			.filter(Boolean);

		fileList.sort();
		hotUpdateManifest.sort((a, b) => (a.name > b.name ? 1 : -1));
		hotUpdateFile.sort((a, b) => (a.name > b.name ? 1 : -1));

		if (runtime?.javascript) {
			runtime.javascript.outdatedModules.sort();
			runtime.javascript.updatedModules.sort();
			runtime.javascript.updatedRuntime.sort();
			runtime.javascript.acceptedModules.sort();
			runtime.javascript.disposedModules.sort();
			for (let value of Object.values(
				runtime.javascript.outdatedDependencies
			)) {
				value.sort();
			}
		}

		let content = `
# ${title}

## Changed Files
${changedFiles.map(i => `- ${i}`).join("\n")}

## Asset Files
${fileList.join("\n")}

## Manifest
${hotUpdateManifest
	.map(
		i => `
### ${i.name}

\`\`\`json
${i.content}
\`\`\`
`
	)
	.join("\n\n")}

## Update

${hotUpdateFile
	.map(
		i => `
### ${i.name}

#### Changed Modules
${i.modules.map(i => `- ${i}`).join("\n")}

#### Changed Runtime Modules
${i.runtime.map(i => `- ${i}`).join("\n")}

#### Changed Content
\`\`\`js
${i.content}
\`\`\`
`
	)
	.join("\n\n")}


${
	runtime
		? `
## Runtime
### Status

\`\`\`txt
${runtime.statusPath.join(" => ")}
\`\`\`

${
	runtime.javascript
		? `

### JavaScript

#### Outdated

Outdated Modules:
${runtime.javascript.outdatedModules.map(i => `- ${i}`).join("\n")}


Outdated Dependencies:
\`\`\`json
${JSON.stringify(runtime.javascript.outdatedDependencies || {}, null, 2)}
\`\`\`

#### Updated

Updated Modules:
${runtime.javascript.updatedModules.map(i => `- ${i}`).join("\n")}

Updated Runtime:
${runtime.javascript.updatedRuntime.map(i => `- \`${i}\``).join("\n")}


#### Callback

Accepted Callback:
${runtime.javascript.acceptedModules.map(i => `- ${i}`).join("\n")}

Disposed Callback:
${runtime.javascript.disposedModules.map(i => `- ${i}`).join("\n")}
`
		: ""
}

`
		: ""
}

				`.trim();

		env.expect(escapeEOL(content)).toMatchFileSnapshot(snapshotPath);
	}
}
