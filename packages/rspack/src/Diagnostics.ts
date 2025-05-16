import util from "node:util";
import { Diagnostics } from "@rspack/binding";
import { RspackError } from "./RspackError";

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
		unshift(...items: RspackError[]): number {
			adm.spliceWithArray(0, 0, items);
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
		flatMap<U, This = undefined>(
			callbackfn: (
				this: This,
				value: RspackError,
				index: number,
				array: RspackError[]
			) => U | ReadonlyArray<U>,
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
			[].reduce;
			return adm.values().map(callbackfn, thisArg);
		},
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
			if (typeof name === "string" && !isNaN(name as any)) {
				return adm.get(parseInt(name));
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
			if (typeof name === "symbol" || isNaN(name as any)) {
				target[name] = value;
			} else {
				// numeric string
				adm.set(parseInt(name), value);
			}
			return true;
		}
	});

	adm[$proxy] = proxy;
	return proxy;
}
