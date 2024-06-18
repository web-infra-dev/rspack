import {
	type JsChunk,
	type JsCompilation,
	__chunk_group_inner_get_chunk_group,
	__chunk_inner_can_be_initial,
	__chunk_inner_get_all_async_chunks,
	__chunk_inner_get_all_initial_chunks,
	__chunk_inner_get_all_referenced_chunks,
	__chunk_inner_has_runtime,
	__chunk_inner_is_only_initial
} from "@rspack/binding";

import { Compilation } from ".";
import { ChunkGroup } from "./ChunkGroup";
import { compareChunkGroupsByIndex } from "./util/comparators";

export class Chunk {
	#inner: JsChunk;
	#inner_compilation: JsCompilation;

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

	static __from_binding(chunk: JsChunk, compilation: Compilation): Chunk;
	static __from_binding(chunk: JsChunk, compilation: JsCompilation): Chunk;
	static __from_binding(
		chunk: JsChunk,
		compilation: Compilation | JsCompilation
	) {
		if (compilation instanceof Compilation) {
			return new Chunk(chunk, compilation.__internal_getInner());
		}
		return new Chunk(chunk, compilation);
	}

	constructor(chunk: JsChunk, compilation: JsCompilation) {
		this.#inner = chunk;
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
			this.#inner.__inner_ukey,
			this.#inner_compilation
		);
	}

	canBeInitial() {
		return __chunk_inner_can_be_initial(
			this.#inner.__inner_ukey,
			this.#inner_compilation
		);
	}

	hasRuntime() {
		return __chunk_inner_has_runtime(
			this.#inner.__inner_ukey,
			this.#inner_compilation
		);
	}

	get groupsIterable(): Iterable<ChunkGroup> {
		const chunk_groups = this.#inner.__inner_groups.map(ukey => {
			const cg = __chunk_group_inner_get_chunk_group(
				ukey as number,
				this.#inner_compilation
			);
			return ChunkGroup.__from_binding(cg, this.#inner_compilation);
		});
		chunk_groups.sort(compareChunkGroupsByIndex);
		return new Set(chunk_groups);
	}

	getAllAsyncChunks(): Iterable<Chunk> {
		return new Set(
			__chunk_inner_get_all_async_chunks(
				this.#inner.__inner_ukey,
				this.#inner_compilation
			).map(c => Chunk.__from_binding(c, this.#inner_compilation))
		);
	}

	getAllInitialChunks(): Iterable<Chunk> {
		return new Set(
			__chunk_inner_get_all_initial_chunks(
				this.#inner.__inner_ukey,
				this.#inner_compilation
			).map(c => Chunk.__from_binding(c, this.#inner_compilation))
		);
	}

	getAllReferencedChunks(): Iterable<Chunk> {
		return new Set(
			__chunk_inner_get_all_referenced_chunks(
				this.#inner.__inner_ukey,
				this.#inner_compilation
			).map(c => Chunk.__from_binding(c, this.#inner_compilation))
		);
	}

	__internal_innerUkey() {
		return this.#inner.__inner_ukey;
	}
}
