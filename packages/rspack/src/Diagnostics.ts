import * as binding from "@rspack/binding";
import { RspackError } from "./RspackError";

/*
 * functions that do alter the internal structure of the array, (based on lib.es6.d.ts)
 * since these functions alter the inner structure of the array, the have side effects.
 * Because the have side effects, they should not be used in computed function,
 * and for that reason the do not call dependencyState.notifyObserved
 */
Object.defineProperty(binding.Diagnostics.prototype, "splice", {
	enumerable: true,
	configurable: true,
	value(
		this: binding.Diagnostics,
		index: number,
		deleteCount?: number,
		...newItems: RspackError[]
	) {
		switch (arguments.length) {
			case 0:
				return [];
			case 1:
				return this.spliceWithArray(index, this.length);
			case 2:
				return this.spliceWithArray(index, deleteCount);
		}
		return this.spliceWithArray(index, deleteCount, newItems);
	}
});

Object.defineProperty(binding.Diagnostics.prototype, "push", {
	enumerable: true,
	configurable: true,
	value(this: binding.Diagnostics, ...newItems: RspackError[]) {
		this.spliceWithArray(this.length, 0, newItems);
		return this.length;
	}
});

Object.defineProperty(binding.Diagnostics.prototype, "pop", {
	enumerable: true,
	configurable: true,
	value(this: binding.Diagnostics) {
		return this.splice(Math.max(this.length - 1, 0), 1)[0];
	}
});

Object.defineProperty(binding.Diagnostics.prototype, "shift", {
	enumerable: true,
	configurable: true,
	value(this: binding.Diagnostics) {
		return this.splice(0, 1)[0];
	}
});

Object.defineProperty(binding.Diagnostics.prototype, "unshift", {
	enumerable: true,
	configurable: true,
	value(this: binding.Diagnostics, ...items: RspackError[]) {
		this.spliceWithArray(0, 0, items);
		return this.length;
	}
});

declare module "@rspack/binding" {
	interface Diagnostics {
		splice(
			index: number,
			deleteCount?: number,
			...newItems: RspackError[]
		): RspackError[];
		push(...newItems: RspackError[]): number;
		pop(): RspackError | undefined;
		shift(): RspackError | undefined;
		unshift(...newItems: RspackError[]): number;
	}
}

export default binding.Diagnostics;
