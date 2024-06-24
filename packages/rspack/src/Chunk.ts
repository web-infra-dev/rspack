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
	#innerCompilation: JsCompilation;

	name?: Readonly<string>;
	id?: Readonly<string>;
	ids: ReadonlyArray<string>;
	idNameHints: ReadonlyArray<string>;
	filenameTemplate?: Readonly<string>;
	cssFilenameTemplate?: Readonly<string>;
	files: ReadonlySet<string>;
	runtime: ReadonlySet<string>;
	hash?: Readonly<string>;
	contentHash: Readonly<Record<string, string>>;
	renderedHash?: Readonly<string>;
	chunkReason?: Readonly<string>;
	auxiliaryFiles: ReadonlySet<string>;

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
		this.#innerCompilation = compilation;

		this.name = chunk.name;
		this.id = chunk.id;
		this.ids = chunk.ids;
		this.idNameHints = chunk.idNameHints;
		this.filenameTemplate = chunk.filenameTemplate;
		this.cssFilenameTemplate = chunk.cssFilenameTemplate;
		this.files = new Set(chunk.files);
		this.runtime = new Set(chunk.runtime);
		this.hash = chunk.hash;
		this.contentHash = chunk.contentHash;
		this.renderedHash = chunk.renderedHash;
		this.chunkReason = chunk.chunkReason;
		this.auxiliaryFiles = new Set(chunk.auxiliaryFiles);
	}

	isOnlyInitial() {
		return __chunk_inner_is_only_initial(
			this.#inner.__inner_ukey,
			this.#innerCompilation
		);
	}

	canBeInitial() {
		return __chunk_inner_can_be_initial(
			this.#inner.__inner_ukey,
			this.#innerCompilation
		);
	}

	hasRuntime() {
		return __chunk_inner_has_runtime(
			this.#inner.__inner_ukey,
			this.#innerCompilation
		);
	}

	get groupsIterable(): Iterable<ChunkGroup> {
		const chunk_groups = this.#inner.__inner_groups.map(ukey => {
			const cg = __chunk_group_inner_get_chunk_group(
				ukey as number,
				this.#innerCompilation
			);
			return ChunkGroup.__from_binding(cg, this.#innerCompilation);
		});
		chunk_groups.sort(compareChunkGroupsByIndex);
		return new Set(chunk_groups);
	}

	getAllAsyncChunks(): Iterable<Chunk> {
		return new Set(
			__chunk_inner_get_all_async_chunks(
				this.#inner.__inner_ukey,
				this.#innerCompilation
			).map(c => Chunk.__from_binding(c, this.#innerCompilation))
		);
	}

	getAllInitialChunks(): Iterable<Chunk> {
		return new Set(
			__chunk_inner_get_all_initial_chunks(
				this.#inner.__inner_ukey,
				this.#innerCompilation
			).map(c => Chunk.__from_binding(c, this.#innerCompilation))
		);
	}

	getAllReferencedChunks(): Iterable<Chunk> {
		return new Set(
			__chunk_inner_get_all_referenced_chunks(
				this.#inner.__inner_ukey,
				this.#innerCompilation
			).map(c => Chunk.__from_binding(c, this.#innerCompilation))
		);
	}

	/**
	 * Note: This is not a webpack public API, maybe removed in future.
	 *
	 * @internal
	 */
	__internal__innerUkey() {
		return this.#inner.__inner_ukey;
	}
}
