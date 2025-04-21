import { Chunks } from "@rspack/binding";
import { Chunk } from "./Chunk";

Object.defineProperty(Chunks.prototype, "values", {
	enumerable: true,
	configurable: true,
	value(this: Chunks): SetIterator<Chunk> {
		return this._values()
			.map(binding => Chunk.__from_binding(binding))
			.values();
	}
});

Object.defineProperty(Chunks.prototype, Symbol.iterator, {
	enumerable: true,
	configurable: true,
	value(this: Chunks): SetIterator<Chunk> {
		return this.values();
	}
});

Object.defineProperty(Chunks.prototype, "keys", {
	enumerable: true,
	configurable: true,
	value(this: Chunks): SetIterator<Chunk> {
		return this.values();
	}
});

Object.defineProperty(Chunks.prototype, "forEach", {
	enumerable: true,
	configurable: true,
	value(
		callbackfn: (value: Chunk, value2: Chunk, set: ReadonlySet<Chunk>) => void,
		thisArg?: any
	): void {
		return this.values().forEach(callbackfn, thisArg);
	}
});

Object.defineProperty(Chunks.prototype, "has", {
	enumerable: true,
	configurable: true,
	value(this: Chunks, value: Chunk): boolean {
		return this._has(Chunk.__to_binding(value));
	}
});

declare module "@rspack/binding" {
	interface Chunks {
		[Symbol.iterator](): SetIterator<Chunk>;
		entries(): SetIterator<[Chunk, Chunk]>;
		values(): SetIterator<Chunk>;
		keys(): SetIterator<Chunk>;
		forEach(
			callbackfn: (
				value: Chunk,
				value2: Chunk,
				set: ReadonlySet<Chunk>
			) => void,
			thisArg?: any
		): void;
		has(value: Chunk): boolean;
	}
}

export default Chunks;
