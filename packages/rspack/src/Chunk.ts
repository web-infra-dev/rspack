import {
	__chunk_inner_can_be_initial,
	__chunk_inner_has_runtime,
	__chunk_inner_is_only_initial,
	type JsChunk,
	type JsCompilation
} from "@rspack/binding";

export class Chunk implements JsChunk {
	#inner_chunk: JsChunk;
	#inner_compilation: JsCompilation;

	// @ts-expect-error should not use inner_ukey in js side
	__inner_ukey: never;

	name?: string;
	id?: string;
	ids: Array<string>;
	idNameHints: Array<string>;
	filenameTemplate?: string;
	cssFilenameTemplate?: string;
	files: Array<string>;
	runtime: Array<string>;
	hash?: string;
	contentHash: Record<string, string>;
	renderedHash?: string;
	chunkReasons: Array<string>;
	auxiliaryFiles: Array<string>;

	static __from_binding(chunk: JsChunk, compilation: JsCompilation) {
		return new Chunk(chunk, compilation);
	}

	// Should not construct by user
	private constructor(chunk: JsChunk, compilation: JsCompilation) {
		this.#inner_chunk = chunk;
		this.#inner_compilation = compilation;

		this.name = chunk.name;
		this.id = chunk.id;
		this.ids = chunk.ids;
		this.idNameHints = chunk.idNameHints;
		this.filenameTemplate = chunk.filenameTemplate;
		this.cssFilenameTemplate = chunk.cssFilenameTemplate;
		this.files = chunk.files;
		this.runtime = chunk.runtime;
		this.hash = chunk.hash;
		this.contentHash = chunk.contentHash;
		this.renderedHash = chunk.renderedHash;
		this.chunkReasons = chunk.chunkReasons;
		this.auxiliaryFiles = chunk.auxiliaryFiles;
	}

	isOnlyInitial() {
		return __chunk_inner_is_only_initial(
			this.#inner_chunk,
			this.#inner_compilation
		);
	}

	canBeInitial() {
		return __chunk_inner_can_be_initial(
			this.#inner_chunk,
			this.#inner_compilation
		);
	}

	hasRuntime() {
		return __chunk_inner_has_runtime(
			this.#inner_chunk,
			this.#inner_compilation
		);
	}
}
