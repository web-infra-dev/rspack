import type { JsCompatSource } from "@rspack/binding";
import { CompatSource, RawSource, Source } from "webpack-sources";

import { isNil } from "./index";

class JsSource extends Source {
	static __from_binding(source: JsCompatSource): Source {
		if (source.isRaw) {
			return new RawSource(
				// @ts-expect-error: webpack-sources can accept buffer as source, see: https://github.com/webpack/webpack-sources/blob/9f98066311d53a153fdc7c633422a1d086528027/lib/RawSource.js#L12
				source.isBuffer ? source.source : source.source.toString("utf-8")
			);
		}

		if (!source.map) {
			return new RawSource(source.source.toString("utf-8"));
		}

		return new CompatSource({
			source() {
				return source.source.toString("utf-8");
			},
			buffer() {
				return source.source;
			},
			map(_) {
				if (source.map) {
					return JSON.parse(source.map.toString("utf-8"));
				}

				return null;
			}
		});
	}

	static __to_binding(source: Source) {
		const sourceSource = source.source();
		const isBuffer = Buffer.isBuffer(sourceSource);

		if (source instanceof RawSource) {
			return {
				source: source.buffer(),
				isRaw: true,
				isBuffer
			};
		}

		const buffer =
			source.buffer?.() ??
			(isBuffer
				? sourceSource
				: sourceSource instanceof ArrayBuffer
					? arrayBufferToBuffer(sourceSource)
					: Buffer.from(sourceSource));
		const map = JSON.stringify(
			source.map?.({
				columns: true
			})
		);

		return {
			source: buffer,
			map: isNil(map) ? map : Buffer.from(map),
			isRaw: false,
			isBuffer
		};
	}
}

function arrayBufferToBuffer(ab: ArrayBuffer) {
	const buf = Buffer.alloc(ab.byteLength);
	const view = new Uint8Array(ab);
	for (let i = 0; i < buf.length; ++i) {
		buf[i] = view[i];
	}
	return buf;
}

export { JsSource };
