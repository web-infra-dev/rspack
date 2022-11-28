export interface Output {
	path?: string;
	publicPath?: string;
	assetModuleFilename?: string;
	filename?: string;
	chunkFilename?: string;
	uniqueName?: string;
	hashFunction?: string;
	cssFilename?: string;
	cssChunkFilename?: string;
}

// TODO: removed optional
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
	cssFilename?: string;
	cssChunkFilename?: string;
}

export function resolveOutputOptions(output: Output = {}): ResolvedOutput {
	return {
		...output
	};
}
