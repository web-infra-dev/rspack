import {
	type BuiltinPlugin,
	BuiltinPluginName,
	type RawShareContainerPluginOptions
} from "@rspack/binding";
import {
	createBuiltinPlugin,
	RspackBuiltinPlugin
} from "../builtin-plugin/base";
import type { Compilation } from "../Compilation";
import type { Compiler } from "../Compiler";
import { encodeName } from "./utils";

export type ShareContainerPluginOptions = {
	mfName: string;
	shareName: string;
	version: string;
	request: string;
};

function assert(condition: any, msg: string): asserts condition {
	if (!condition) {
		throw new Error(msg);
	}
}

const HOT_UPDATE_SUFFIX = ".hot-update";

export class ShareContainerPlugin extends RspackBuiltinPlugin {
	name = BuiltinPluginName.ShareContainerPlugin;
	filename = "";
	_options: RawShareContainerPluginOptions;

	constructor(options: ShareContainerPluginOptions) {
		super();
		const version = options.version || "0.0.0";
		const containerName = encodeName(
			`${options.mfName}_${options.shareName}_${version}`
		);
		const globalName = encodeName(
			`${options.mfName}_${options.shareName}_${version}_global`
		);
		const fileName = `independent-share/${options.shareName}@${version}/${options.mfName}.container.js`;

		this._options = {
			name: containerName,
			shareName: options.shareName,
			request: options.request,
			version,
			globalName,
			fileName
		};
	}
	getData() {
		return this.name;
	}

	raw(_compiler: Compiler): BuiltinPlugin {
		return createBuiltinPlugin(this.name, this._options);
	}

	apply(compiler: Compiler) {
		const { shareName } = this._options;
		compiler.hooks.thisCompilation.tap(
			this.name,
			(compilation: Compilation, { normalModuleFactory }) => {
				compilation.hooks.processAssets.tapPromise(
					{
						name: "getShareContainerFile"
					},
					async () => {
						const remoteEntryPoint = compilation.entrypoints.get(shareName);
						assert(
							remoteEntryPoint,
							`Can not get shared ${shareName} entryPoint!`
						);
						const remoteEntryNameChunk = compilation.namedChunks.get(shareName);
						assert(
							remoteEntryNameChunk,
							`Can not get shared ${shareName} chunk!`
						);

						const files = Array.from(
							remoteEntryNameChunk.files as Iterable<string>
						).filter(
							(f: string) =>
								!f.includes(HOT_UPDATE_SUFFIX) && !f.endsWith(".css")
						);
						assert(
							files.length > 0,
							`no files found for shared ${shareName} chunk`
						);
						assert(
							files.length === 1,
							`shared ${shareName} chunk should not have multiple files!, current files: ${files.join(
								","
							)}`
						);
						this.filename = files[0];
					}
				);
			}
		);
	}
}
