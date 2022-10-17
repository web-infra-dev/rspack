import path from "path";
import { ResolvedContext } from "./context";

export interface Output {
	path?: string;
	publicPath?: string;
	assetModuleFilename?: string;
	filename?: string;
	chunkFilename?: string;
	uniqueName?: string;
}

// TODO: fix it
export interface ResolvedOutput {
	path?: string;
	publicPath?: string;
	assetModuleFilename?: string;
	filename?: string;
	chunkFilename?: string;
	uniqueName?: string;
}

export function resolveOutputOptions(output: Output = {}, context: ResolvedContext): ResolvedOutput {
	return {
		path: output.path ?? path.join(context, "dist"),
		publicPath: output.publicPath,
		chunkFilename: output.chunkFilename,
		filename: output.publicPath,
		assetModuleFilename: output.assetModuleFilename,
		uniqueName: output.uniqueName
	};
}
