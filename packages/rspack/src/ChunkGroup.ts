import {
	__chunk_group_inner_get_chunk_group,
	type JsChunkGroup,
	type JsCompilation
} from "@rspack/binding";

export class ChunkGroup {
	#inner: JsChunkGroup;
	#inner_compilation: JsCompilation;

	static __from_binding(chunk: JsChunkGroup, compilation: JsCompilation) {
		return new ChunkGroup(chunk, compilation);
	}

	protected constructor(inner: JsChunkGroup, compilation: JsCompilation) {
		this.#inner = inner;
		this.#inner_compilation = compilation;
	}

	getFiles(): string[] {
		const files = new Set<string>();

		for (const chunk of this.#inner.chunks) {
			for (const file of chunk.files) {
				files.add(file);
			}
		}

		return Array.from(files);
	}

	getParents(): ChunkGroup[] {
		return this.#inner.__inner_parents.map(parent => {
			const cg = __chunk_group_inner_get_chunk_group(
				parent,
				this.#inner_compilation
			);
			return ChunkGroup.__from_binding(cg, this.#inner_compilation);
		});
	}

	get index(): number | undefined {
		return this.#inner.index;
	}

	get name(): string | undefined {
		return this.#inner.name;
	}

	__internal_inner_ukey() {
		return this.#inner.__inner_ukey;
	}

	__internal_inner_compilation() {
		return this.#inner_compilation;
	}
}
