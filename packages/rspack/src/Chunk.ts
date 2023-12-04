import {
	__chunk_group_inner_get_chunk_group,
	__chunk_inner_can_be_initial,
	__chunk_inner_get_chunk_modules,
	__chunk_inner_has_runtime,
	__chunk_inner_is_only_initial,
	type JsChunk,
	type JsCompilation
} from "@rspack/binding";
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

	static __from_binding(chunk: JsChunk, compilation: JsCompilation) {
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

	get groupsIterable(): Set<ChunkGroup> {
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

	__internal_inner_ukey() {
		return this.#inner.__inner_ukey;
	}
}
