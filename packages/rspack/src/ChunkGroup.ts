import {
	type JsChunkGroup,
	type JsCompilation,
	__chunk_group_inner_get_chunk_group
} from "@rspack/binding";

import { Chunk } from "./Chunk";

export class ChunkGroup {
	#inner: JsChunkGroup;
	#innerCompilation: JsCompilation;

	static __from_binding(chunk: JsChunkGroup, compilation: JsCompilation) {
		return new ChunkGroup(chunk, compilation);
	}

	protected constructor(inner: JsChunkGroup, compilation: JsCompilation) {
		this.#inner = inner;
		this.#innerCompilation = compilation;
	}

	getFiles(): ReadonlyArray<string> {
		const files = new Set<string>();

		for (const chunk of this.#inner.chunks) {
			for (const file of chunk.files) {
				files.add(file);
			}
		}

		return Array.from(files);
	}

	getParents(): ReadonlyArray<ChunkGroup> {
		return this.#inner.__inner_parents.map(parent => {
			const cg = __chunk_group_inner_get_chunk_group(
				parent,
				this.#innerCompilation
			);
			return ChunkGroup.__from_binding(cg, this.#innerCompilation);
		});
	}

	get chunks(): ReadonlyArray<Chunk> {
		return this.#inner.chunks.map(c =>
			Chunk.__from_binding(c, this.#innerCompilation)
		);
	}

	get index(): Readonly<number | undefined> {
		return this.#inner.index;
	}

	get name(): Readonly<string | undefined> {
		return this.#inner.name;
	}

	/**
	 * Note: This is not a webpack public API, maybe removed in future.
	 *
	 * @internal
	 */
	__internal__innerUkey() {
		return this.#inner.__inner_ukey;
	}

	/**
	 * Note: This is not a webpack public API, maybe removed in future.
	 *
	 * @internal
	 */
	__internal__innerCompilation() {
		return this.#innerCompilation;
	}
}
