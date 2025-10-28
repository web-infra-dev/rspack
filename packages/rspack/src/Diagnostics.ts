import util from "node:util";
import type { Diagnostics } from "@rspack/binding";
import type { RspackError } from "./RspackError";

const $proxy = Symbol.for("proxy");

export function createDiagnosticArray(
	adm: Diagnostics & { [$proxy]?: RspackError[] }
): RspackError[] {
	if ($proxy in adm) {
		return adm[$proxy] as RspackError[];
	}

	const array: RspackError[] & {
		[util.inspect.custom]?: () => RspackError[];
		[key: string | symbol]: any;
	} = [];
	array[util.inspect.custom] = () => {
		return adm.values();
	};

	const splice = function splice(
		index: number,
		deleteCount?: number,
		...newItems: RspackError[]
	): RspackError[] {
		switch (arguments.length) {
			case 0:
				return [];
			case 1:
				return adm.spliceWithArray(index, adm.length);
			case 2:
				return adm.spliceWithArray(index, deleteCount);
		}

		return adm.spliceWithArray(index, deleteCount, newItems);
	};

	const arrayExtensions: Record<string | symbol, any> = {
		[Symbol.iterator](): ArrayIterator<RspackError> {
			return adm.values().values();
		},
		splice,
		push(...newItems: RspackError[]): number {
			adm.spliceWithArray(adm.length, 0, newItems);
			return adm.length;
		},
		pop(): RspackError | undefined {
			return splice(Math.max(adm.length - 1, 0), 1)[0];
		},
		shift(): RspackError | undefined {
			return splice(0, 1)[0];
		},
		unshift(...newItems: RspackError[]): number {
			adm.spliceWithArray(0, 0, newItems);
			return adm.length;
		},
		reverse(): RspackError[] {
			return adm.values().reverse();
		},
		sort(
			this: RspackError[],
			compareFn?: (a: RspackError, b: RspackError) => number
		): RspackError[] {
			const copy = adm.values();
			copy.sort(compareFn);
			adm.spliceWithArray(0, adm.length, copy);
			return this;
		},
		at(index: number): RspackError | undefined {
			return adm.get(index);
		},
		concat(...items: RspackError[]): RspackError[] {
			[].includes;
			return adm.values().concat(...items);
		},
		flat(): RspackError[] {
			return adm.values();
		},

		// map
		every<S extends RspackError>(
			predicate: (
				value: RspackError,
				index: number,
				array: RspackError[]
			) => value is S,
			thisArg?: any
		): this is S[] {
			return adm.values().every(predicate, thisArg);
		},
		filter<S extends RspackError>(
			predicate: (
				value: RspackError,
				index: number,
				array: RspackError[]
			) => value is S,
			thisArg?: any
		): S[] {
			return adm.values().filter(predicate, thisArg);
		},
		find(
			predicate: (
				value: RspackError,
				index: number,
				obj: RspackError[]
			) => unknown,
			thisArg?: any
		): RspackError | undefined {
			return adm.values().find(predicate, thisArg);
		},
		findIndex(
			predicate: (
				value: RspackError,
				index: number,
				obj: RspackError[]
			) => unknown,
			thisArg?: any
		): number {
			return adm.values().findIndex(predicate, thisArg);
		},
		flatMap<U, This = undefined>(
			callbackfn: (
				this: This,
				value: RspackError,
				index: number,
				array: RspackError[]
			) => U | readonly U[],
			thisArg?: This
		): U[] {
			return adm.values().flatMap(callbackfn, thisArg);
		},
		forEach(
			callbackfn: (
				value: RspackError,
				index: number,
				array: RspackError[]
			) => void,
			thisArg?: any
		): void {
			adm.values().forEach(callbackfn, thisArg);
		},
		map<U>(
			callbackfn: (
				value: RspackError,
				index: number,
				array: RspackError[]
			) => U,
			thisArg?: any
		): U[] {
			return adm.values().map(callbackfn, thisArg);
		},
		slice(start?: number, end?: number): RspackError[] {
			return adm.values().slice(start, end);
		},

		// reduce
		reduce(
			callbackfn: (
				previousValue: RspackError,
				currentValue: RspackError,
				currentIndex: number,
				array: RspackError[]
			) => RspackError,
			initialValue: RspackError
		): RspackError {
			return adm.values().reduce(callbackfn, initialValue);
		},
		reduceRight(
			callbackfn: (
				previousValue: RspackError,
				currentValue: RspackError,
				currentIndex: number,
				array: RspackError[]
			) => RspackError,
			initialValue: RspackError
		): RspackError {
			return adm.values().reduceRight(callbackfn, initialValue);
		}
	};

	const proxy = new Proxy(array, {
		get(target, name) {
			if (name === "length") {
				return adm.length;
			}
			if (typeof name === "string" && !Number.isNaN(Number.parseInt(name))) {
				return adm.get(Number.parseInt(name));
			}
			if (Object.prototype.hasOwnProperty.call(arrayExtensions, name)) {
				return arrayExtensions[name];
			}
			return target[name];
		},
		set(target, name, value): boolean {
			if (name === "length") {
				throw new Error(
					"The 'length' property is read-only and cannot be assigned a new value."
				);
			}
			if (typeof name === "symbol" || Number.isNaN(Number.parseInt(name))) {
				target[name] = value;
			} else {
				// numeric string
				adm.set(Number.parseInt(name), value);
			}
			return true;
		}
	});

	adm[$proxy] = proxy;
	return proxy;
}
