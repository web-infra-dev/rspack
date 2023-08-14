import type { JsCompatSource } from "@rspack/binding";

import { RawSource, Source } from "webpack-sources";
import { isNil } from "../util/index";

class LazyCompatSource extends Source {
	constructor(private s: JsCompatSource) {
		super();
	}

	source() {
		const source = this.s;

		if (source.isRaw && source.isBuffer) {
			return source.source;
		}

		return source.source.toString("utf-8");
	}

	map() {
		const source = this.s;

		if (source.map) {
			return JSON.parse(source.map.toString("utf-8"));
		}

		return null;
	}
}

let interopMap = new WeakMap<JsCompatSource, Source>();

function toJsSource(source: JsCompatSource): Source {
	let s;
	if ((s = interopMap.get(source))) {
		return s;
	}
	s = new LazyCompatSource(source);
	interopMap.set(source, s);
	return s;
}

function toRustSource(source: Source): JsCompatSource {
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

function arrayBufferToBuffer(ab: ArrayBuffer) {
	const buf = Buffer.alloc(ab.byteLength);
	const view = new Uint8Array(ab);
	for (let i = 0; i < buf.length; ++i) {
		buf[i] = view[i];
	}
	return buf;
}

export { toJsSource, toRustSource };
