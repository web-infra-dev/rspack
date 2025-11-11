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
import type { LibraryOptions } from "../config";
import { encodeName } from "./utils";

export type ShareContainerPluginOptions = {
	mfName: string;
	shareName: string;
	version: string;
	request: string;
	library?: LibraryOptions;
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
	_shareName: string;

	constructor(options: ShareContainerPluginOptions) {
		super();
		const { shareName, mfName, library, request } = options;
		const version = options.version || "0.0.0";
		const globalName = encodeName(`${options.mfName}_${shareName}_${version}`);
		const fileName = `independent-share-${encodeName(mfName)}/${encodeName(shareName)}/${version}/share-entry.js`;
		this._shareName = shareName;
		this._options = {
			name: shareName,
			request: request,
			library: library || {
				type: "var",
				name: globalName
			},
			version,
			fileName
		};
	}
	getData() {
		return this.name;
	}

	raw(compiler: Compiler): BuiltinPlugin {
		const { library } = this._options;
		if (!compiler.options.output.enabledLibraryTypes!.includes(library.type)) {
			compiler.options.output.enabledLibraryTypes!.push(library.type);
		}
		return createBuiltinPlugin(this.name, this._options);
	}

	apply(compiler: Compiler) {
		super.apply(compiler);
		const shareName = this._shareName;
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
