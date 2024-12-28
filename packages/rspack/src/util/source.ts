import type { JsCompatSourceOwned } from "@rspack/binding";
import { RawSource, Source, SourceMapSource } from "webpack-sources";

class JsSource extends Source {
	static __from_binding(source: JsCompatSourceOwned): Source {
		if (Buffer.isBuffer(source.source)) {
			// @ts-expect-error: webpack-sources can accept buffer as source,
			// see: https://github.com/webpack/webpack-sources/blob/9f98066311d53a153fdc7c633422a1d086528027/lib/RawSource.js#L12
			return new RawSource(source.source);
		}
		if (!source.map) {
			return new RawSource(source.source);
		}
		return new SourceMapSource(
			source.source,
			"inmemory://from rust",
			// @ts-expect-error: SourceMapSource can accept string as source map,
			// see: https://github.com/webpack/webpack-sources/blob/9f98066311d53a153fdc7c633422a1d086528027/lib/SourceMapSource.js#L30
			source.map
		);
	}

	static __to_binding(source: Source): JsCompatSourceOwned {
		if (source instanceof RawSource) {
			// @ts-expect-error: The 'isBuffer' method exists on 'RawSource' in 'webpack-sources',
			if (source.isBuffer()) {
				return {
					source: source.buffer()
				};
			}
			return {
				source: source.source()
			};
		}

		const map = JSON.stringify(
			source.map?.({
				columns: true
			})
		);

		const code = source.source();
		return {
			source:
				typeof code === "string" ? code : Buffer.from(code).toString("utf-8"),
			map
		};
	}
}

export { JsSource };
