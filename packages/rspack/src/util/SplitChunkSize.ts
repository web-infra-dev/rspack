import { RawSplitChunkSizes } from "@rspack/binding";

class JsSplitChunkSizes {
	static __to_binding(
		sizes?: number | Record<string, number>
	): number | RawSplitChunkSizes | undefined {
		if (typeof sizes === "number") {
			return sizes;
		} else if (sizes && typeof sizes === "object") {
			const chunkSizes: RawSplitChunkSizes = {
				sizes: sizes
			};
			return chunkSizes;
		}
		return sizes;
	}
}

export { JsSplitChunkSizes };
