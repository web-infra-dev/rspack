import type { JsChunkGroup } from "@rspack/binding";

export class ChunkGroup {
	#inner: JsChunkGroup;

	constructor(inner: JsChunkGroup) {
		this.#inner = inner;
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
}
