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

export function resolveOutputOptions(output: Output = {}): ResolvedOutput {
	return {
		path: output.path,
		publicPath: output.publicPath,
		chunkFilename: output.chunkFilename,
		filename: output.publicPath,
		assetModuleFilename: output.assetModuleFilename,
		uniqueName: output.uniqueName
	};
}
