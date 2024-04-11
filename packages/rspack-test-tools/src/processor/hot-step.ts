import {
	ECompilerType,
	ITestContext,
	ITestEnv,
	TCompilerOptions,
	TCompilerStats
} from "../type";
import path from "path";
import { StatsAsset } from "@rspack/core";
import {
	IRspackHotProcessorOptions,
	RspackHotProcessor,
	TUpdateOptions
} from "./hot";
import fs from "fs-extra";

const escapeLocalName = (str: string) => str.split(/[-<>:"/|?*.]/).join("_");

declare var global: {
	self?: {
		[key: string]: (name: string, modules: Record<string, unknown>) => void;
	};
	updateSnapshot: boolean;
};

const SELF_HANDLER = (
	file: string,
	options: TCompilerOptions<ECompilerType.Rspack>
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

const GET_MODULE_HANDLER = {
	web: SELF_HANDLER,
	"async-node": (file: string): string[] => {
		return Object.keys(require(file).modules) || [];
	},
	webworker: SELF_HANDLER
};

type TSupportTarget = keyof typeof GET_MODULE_HANDLER;

export interface IRspackHotStepProcessorOptions
	extends IRspackHotProcessorOptions {}

export class RspackHotStepProcessor extends RspackHotProcessor {
	private hashes: string[] = [];

	constructor(protected _hotOptions: IRspackHotProcessorOptions) {
		super(_hotOptions);
	}

	async run(env: ITestEnv, context: ITestContext) {
		context.setValue(
			this._options.name,
			"hotUpdateStepChecker",
			(
				hotUpdateContext: TUpdateOptions,
				stats: TCompilerStats<ECompilerType.Rspack>
			) => {
				const statsJson = stats.toJson({ assets: true });
				this.matchStepSnapshot(
					context,
					hotUpdateContext.updateIndex,
					statsJson.assets,
					statsJson.hash!
				);
				this.hashes.push(stats.hash!);
			}
		);
		await super.run(env, context);
	}

	async check(env: ITestEnv, context: ITestContext) {
		const compiler = this.getCompiler(context);
		const stats = compiler.getStats();
		if (!stats || !stats.hash) {
			expect(false);
			return;
		}
		this.hashes.push(stats.hash!);

		const assets = stats.toJson({ assets: true }).assets;
		this.matchStepSnapshot(context, 0, assets, stats.hash);

		await super.check(env, context);
	}

	protected matchStepSnapshot(
		context: ITestContext,
		step: number,
		assets: StatsAsset[] = [],
		hash: string
	) {
		const compiler = this.getCompiler(context);
		const compilerOptions = compiler.getOptions();
		const getModuleHandler =
			GET_MODULE_HANDLER[compilerOptions.target as TSupportTarget];
		expect(typeof getModuleHandler).toBe("function");

		const lastHash = this.hashes[this.hashes.length - 1];
		const snapshotPath = context.getSource(
			`snapshot/${compilerOptions.target}/${step}.snap.txt`
		);
		const title = `Case ${this._options.name}: Step ${step}`;
		const hotUpdateFile: Array<{
			name: string;
			content: string;
			modules: string[];
			runtime: string[];
		}> = [];
		const hotUpdateManifest: Array<{ name: string; content: string }> = [];
		const changedFiles: string[] = require(context.getSource(
			"changed-file.js"
		)).map((i: string) => path.relative(context.getSource(), i));

		const fileList = assets
			.map(i => {
				if (i.name.endsWith("hot-update.js")) {
					const modules = getModuleHandler(
						context.getDist(i.name),
						compilerOptions
					);
					const content = fs.readFileSync(context.getDist(i.name), "utf-8");
					const runtime: string[] = [];
					for (let i of content.matchAll(
						/\/\/ (webpack\/runtime\/[\w_-]+)\s*\n/g
					)) {
						runtime.push(i[1]);
					}
					modules.sort();
					runtime.sort();
					hotUpdateFile.push({
						name: i.name,
						content,
						modules,
						runtime
					});
					return `- Update: ${i.name}, size: ${i.size}`;
				} else if (i.name.endsWith("hot-update.json")) {
					hotUpdateManifest.push({
						name: i.name,
						content: fs.readFileSync(context.getDist(i.name), "utf-8")
					});
					return `- Manifest: ${i.name}, size: ${i.size}`;
				} else if (i.name.endsWith(".js")) {
					return `- Bundle: ${i.name}, size: ${i.size}`;
				}
			})
			.filter(Boolean);

		fileList.sort();
		hotUpdateManifest.sort();
		hotUpdateFile.sort();

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

		`;
		if (lastHash) {
			content = content
				.split(lastHash)
				.join("LAST_HASH")
				.split(hash)
				.join("CURRENT_HASH");
		}

		if (!fs.existsSync(snapshotPath) || global.updateSnapshot) {
			fs.ensureDirSync(path.dirname(snapshotPath));
			fs.writeFileSync(snapshotPath, content, "utf-8");
			return;
		}
		const snapshotContent = fs
			.readFileSync(snapshotPath, "utf-8")
			.replace(/\r\n/g, "\n")
			.trim();
		expect(content).toBe(snapshotContent);
	}
}
