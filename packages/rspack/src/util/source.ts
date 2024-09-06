import type { JsCompatSource } from "@rspack/binding";
import { RawSource, Source, SourceMapSource } from "webpack-sources";

import { isNil } from "./index";

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
		const isBuffer = Buffer.isBuffer(sourceSource);

		if (source instanceof RawSource) {
			return {
				source: source.buffer(),
				isBuffer
			};
		}

		const buffer =
			source.buffer?.() ??
			(isBuffer
				? sourceSource
				: sourceSource instanceof ArrayBuffer
					? Buffer.from(sourceSource)
					: Buffer.from(sourceSource));
		const map = JSON.stringify(
			source.map?.({
				columns: true
			})
		);

		return {
			source: buffer,
			map: isNil(map) ? map : Buffer.from(map),
			isBuffer
		};
	}
}

export { JsSource };
