import type { JsCompatSource } from "@rspack/binding";
import { RawSource, Source, SourceMapSource } from "webpack-sources";

class JsSource extends Source {
	static __from_binding(source: JsCompatSource): Source {
		if (!source.map) {
			return new RawSource(source.source.toString("utf-8"));
		}

		return new SourceMapSource(
			source.source.toString("utf-8"),
			"from rust",
			source.map ? JSON.parse(source.map.toString("utf-8")) : null
		);
	}

	static __to_binding(source: Source) {
		const sourceSource = source.source();

		if (source instanceof RawSource) {
			return {
				source: source.buffer()
			};
		}

		const map = JSON.stringify(
			source.map?.({
				columns: true
			})
		);

		return {
			source: sourceSource,
			map
		};
	}
}

export { JsSource };
