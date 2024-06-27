declare class Hash {
	constructor();

	/**
	 * Update hash {@link https://nodejs.org/api/crypto.html#crypto_hash_update_data_inputencoding}
	 */
	update(data: string | Buffer, inputEncoding?: string): Hash;

	/**
	 * Calculates the digest {@link https://nodejs.org/api/crypto.html#crypto_hash_digest_encoding}
	 */
	digest(encoding?: string): string | Buffer;
}

export type MapOptions = { columns?: boolean; module?: boolean };

export type RawSourceMap = {
	version: number;
	sources: string[];
	names: string[];
	sourceRoot?: string;
	sourcesContent?: string[];
	mappings: string;
	file: string;
};

export abstract class Source {
	size(): number;

	map(options?: MapOptions): RawSourceMap | null;

	sourceAndMap(options?: MapOptions): {
		source: string | Buffer;
		map: Object;
	};

	updateHash(hash: Hash): void;

	source(): string | Buffer;

	buffer(): Buffer;
}

export class RawSource extends Source {
	constructor(source: string | Buffer, convertToString?: boolean);

	isBuffer(): boolean;
}

export class OriginalSource extends Source {
	constructor(source: string | Buffer, name: string);

	getName(): string;
}

export class ReplaceSource extends Source {
	constructor(source: Source, name?: string);

	replace(start: number, end: number, newValue: string, name?: string): void;
	insert(pos: number, newValue: string, name?: string): void;

	getName(): string;
	original(): string;
	getReplacements(): {
		start: number;
		end: number;
		content: string;
		insertIndex: number;
		name: string;
	}[];
}

export class SourceMapSource extends Source {
	constructor(
		source: string | Buffer,
		name: string,
		sourceMap: Object | string | Buffer,
		originalSource?: string | Buffer,
		innerSourceMap?: Object | string | Buffer,
		removeOriginalSource?: boolean
	);

	getArgsAsBuffers(): [
		Buffer,
		string,
		Buffer,
		Buffer | undefined,
		Buffer | undefined,
		boolean
	];
}

export class ConcatSource extends Source {
	constructor(...args: (string | Source)[]);

	getChildren(): Source[];

	add(item: string | Source): void;
	addAllSkipOptimizing(items: Source[]): void;
}

export class PrefixSource extends Source {
	constructor(prefix: string, source: string | Source);

	original(): Source;
	getPrefix(): string;
}

export class CachedSource extends Source {
	constructor(source: Source);
	constructor(source: Source | (() => Source), cachedData?: any);

	original(): Source;
	originalLazy(): Source | (() => Source);
	getCachedData(): any;
}

export class SizeOnlySource extends Source {
	constructor(size: number);
}

interface SourceLike {
	source(): string | Buffer;
}

export class CompatSource extends Source {
	constructor(sourceLike: SourceLike);

	static from(sourceLike: SourceLike): Source;
}
