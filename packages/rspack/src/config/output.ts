export interface Output {
	path?: string;
	publicPath?: string;
	assetModuleFilename?: string;
	filename?: string;
	chunkFilename?: string;
	uniqueName?: string;
	hashFunction?: string;
}

// TODO: fix it
export interface ResolvedOutput {
	path?: string;
	publicPath?: string;
	assetModuleFilename?: string;
	filename?: string;
	chunkFilename?: string;
	uniqueName?: string;
	hashFunction?: string;
	hashDigestLength?: string;
	hashDigest?: string;
	hashSalt?: string;
}

export function resolveOutputOptions(output: Output = {}): ResolvedOutput {
	return {
		...output
	};
}
