type FixedSizeArray<T extends number, U> = T extends 0
	? void[]
	: ReadonlyArray<U> & {
			0: U;
			length: T;
	  };
type Measure<T extends number> = T extends 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8
	? T
	: never;
type Append<T extends any[], U> = {
	0: [U];
	1: [T[0], U];
	2: [T[0], T[1], U];
	3: [T[0], T[1], T[2], U];
	4: [T[0], T[1], T[2], T[3], U];
	5: [T[0], T[1], T[2], T[3], T[4], U];
	6: [T[0], T[1], T[2], T[3], T[4], T[5], U];
	7: [T[0], T[1], T[2], T[3], T[4], T[5], T[6], U];
	8: [T[0], T[1], T[2], T[3], T[4], T[5], T[6], T[7], U];
}[Measure<T["length"]>];
export type AsArray<T> = T extends any[] ? T : [T];

export type Fn<T, R> = (...args: AsArray<T>) => R;
export type FnWithCallback<T, R> = (
	...args: Append<AsArray<T>, InnerCallback<Error, R>>
) => void;

declare class UnsetAdditionalOptions {
	_UnsetAdditionalOptions: true;
}
type IfSet<X> = X extends UnsetAdditionalOptions ? {} : X;

type Callback<E, T> = (error: E | null, result?: T) => void;
type InnerCallback<E, T> = (error?: E | null | false, result?: T) => void;

type FullTap = Tap & {
	type: "sync" | "async" | "promise";
	fn: Function;
};

type Tap = TapOptions & {
	name: string;
};

type TapOptions = {
	before?: string;
	stage?: number;
};

export type Options<AdditionalOptions = UnsetAdditionalOptions> =
	| string
	| (Tap & IfSet<AdditionalOptions>);

export interface HookInterceptor<
	T,
	R,
	AdditionalOptions = UnsetAdditionalOptions
> {
	name?: string;
	tap?: (tap: FullTap & IfSet<AdditionalOptions>) => void;
	call?: (...args: any[]) => void;
	loop?: (...args: any[]) => void;
	error?: (err: Error) => void;
	result?: (result: R) => void;
	done?: () => void;
	register?: (
		tap: FullTap & IfSet<AdditionalOptions>
	) => FullTap & IfSet<AdditionalOptions>;
}

type ArgumentNames<T extends any[]> = FixedSizeArray<T["length"], string>;

class Hook<T, R, AdditionalOptions = UnsetAdditionalOptions> {
	args?: ArgumentNames<AsArray<T>>;
	name?: string;
	taps: (FullTap & IfSet<AdditionalOptions>)[];
	interceptors: HookInterceptor<T, R, AdditionalOptions>[];

	constructor(args?: ArgumentNames<AsArray<T>>, name?: string) {
		this.args = args;
		this.name = name;
		this.taps = [];
		this.interceptors = [];
	}

	intercept(interceptor: HookInterceptor<T, R, AdditionalOptions>) {
		this.interceptors.push(Object.assign({}, interceptor));
		if (interceptor.register) {
			for (let i = 0; i < this.taps.length; i++) {
				this.taps[i] = interceptor.register(this.taps[i]);
			}
		}
	}

	_runRegisterInterceptors(
		options: FullTap & IfSet<AdditionalOptions>
	): FullTap & IfSet<AdditionalOptions> {
		for (const interceptor of this.interceptors) {
			if (interceptor.register) {
				const newOptions = interceptor.register(options);
				if (newOptions !== undefined) {
					options = newOptions;
				}
			}
		}
		return options;
	}

	_runCallInterceptors(...args: any[]) {
		for (const interceptor of this.interceptors) {
			if (interceptor.call) {
				interceptor.call(...args);
			}
		}
	}

	_runErrorInterceptors(e: Error) {
		for (const interceptor of this.interceptors) {
			if (interceptor.error) {
				interceptor.error(e);
			}
		}
	}

	_runTapInterceptors(tap: FullTap & IfSet<AdditionalOptions>) {
		for (const interceptor of this.interceptors) {
			if (interceptor.tap) {
				interceptor.tap(tap);
			}
		}
	}

	_runDoneInterceptors() {
		for (const interceptor of this.interceptors) {
			if (interceptor.done) {
				interceptor.done();
			}
		}
	}

	isUsed() {
		return this.taps.length > 0 || this.interceptors.length > 0;
	}

	queryStageRange([from, to]: StageRange) {
		const tapsInRange = [];
		for (let tap of this.taps) {
			const stage = tap.stage ?? 0;
			if (from < stage && stage <= to) {
				tapsInRange.push(tap);
			}
		}
		return tapsInRange;
	}

	callAsyncStageRange(
		stageRange: StageRange,
		...args: Append<AsArray<T>, Callback<Error, R>>
	) {
		throw new Error("Hook should implement there own _callAsyncStageRange");
	}

	callAsync(...args: Append<AsArray<T>, Callback<Error, R>>): void {
		return this.callAsyncStageRange(allStageRange, ...args);
	}

	promiseStageRange(stageRange: StageRange, ...args: AsArray<T>): Promise<R> {
		return new Promise((resolve, reject) => {
			// @ts-expect-error
			this.callAsyncStageRange(stageRange, ...args, (e, r) => {
				if (e) return reject(e);
				return resolve(r);
			});
		});
	}

	promise(...args: AsArray<T>): Promise<R> {
		return this.promiseStageRange(allStageRange, ...args);
	}

	tap(options: Options<AdditionalOptions>, fn: Fn<T, R>) {
		this._tap("sync", options, fn);
	}

	_tap(
		type: "sync" | "async" | "promise",
		options: Options<AdditionalOptions>,
		fn: Function
	) {
		if (typeof options === "string") {
			options = {
				name: options.trim()
			} as Tap & IfSet<AdditionalOptions>;
		} else if (typeof options !== "object" || options === null) {
			throw new Error("Invalid tap options");
		}
		if (typeof options.name !== "string" || options.name === "") {
			throw new Error("Missing name for tap");
		}
		let insert: FullTap & IfSet<AdditionalOptions> = Object.assign(
			{ type, fn },
			options
		);
		insert = this._runRegisterInterceptors(insert);
		this._insert(insert);
	}

	_insert(item: FullTap & IfSet<AdditionalOptions>) {
		let before;
		if (typeof item.before === "string") {
			before = new Set([item.before]);
		} else if (Array.isArray(item.before)) {
			before = new Set(item.before);
		}
		let stage = 0;
		if (typeof item.stage === "number") {
			stage = item.stage;
		}
		let i = this.taps.length;
		while (i > 0) {
			i--;
			const x = this.taps[i];
			this.taps[i + 1] = x;
			const xStage = x.stage || 0;
			if (before) {
				if (before.has(x.name)) {
					before.delete(x.name);
					continue;
				}
				if (before.size > 0) {
					continue;
				}
			}
			if (xStage > stage) {
				continue;
			}
			i++;
			break;
		}
		this.taps[i] = item;
	}
}

export type StageRange = readonly [number, number];
const minStage = -Infinity;
const maxStage = Infinity;
const allStageRange = [minStage, maxStage] as const;

export class SyncHook<
	T,
	R = void,
	AdditionalOptions = UnsetAdditionalOptions
> extends Hook<T, R, AdditionalOptions> {
	callAsyncStageRange(
		[from, to]: StageRange,
		...args: Append<AsArray<T>, Callback<Error, R>>
	) {
		const args2 = [...args];
		const cb = args2.pop() as Callback<Error, R>;
		if (from === minStage) {
			this._runCallInterceptors(...args2);
		}
		for (let tap of this.taps) {
			const stage = tap.stage ?? 0;
			if (from < stage && stage <= to) {
				this._runTapInterceptors(tap);
				try {
					tap.fn(...args2);
				} catch (e) {
					const err = e as Error;
					this._runErrorInterceptors(err);
					return cb(err);
				}
			}
		}
		if (to === maxStage) {
			this._runDoneInterceptors();
			cb(null);
		}
	}

	call(...args: AsArray<T>): R {
		return this.callStageRange(allStageRange, ...args);
	}

	/**
	 * call a range of taps, from < stage <= to, (from, to]
	 */
	callStageRange(stageRange: StageRange, ...args: AsArray<T>): R {
		let result, error;
		// @ts-expect-error
		this.callAsyncStageRange(stageRange, ...args, (e, r) => {
			error = e;
			result = r;
		});
		if (error) {
			throw error;
		}
		return result as R;
	}

	tapAsync(): never {
		throw new Error("tapAsync is not supported on a SyncHook");
	}

	tapPromise(): never {
		throw new Error("tapPromise is not supported on a SyncHook");
	}
}

export class AsyncParallelHook<
	T,
	AdditionalOptions = UnsetAdditionalOptions
> extends Hook<T, void, AdditionalOptions> {
	callAsyncStageRange(
		[from, to]: StageRange,
		...args: Append<AsArray<T>, Callback<Error, void>>
	) {
		const args2 = [...args];
		const cb = args2.pop() as Callback<Error, void>;
		if (from === minStage) {
			this._runCallInterceptors(...args2);
		}
		const done = () => {
			this._runDoneInterceptors();
			cb(null);
		};
		const error = (e: Error) => {
			this._runErrorInterceptors(e);
			cb(e);
		};
		const tapsInRange = [];
		for (let tap of this.taps) {
			const stage = tap.stage ?? 0;
			if (from < stage && stage <= to) {
				tapsInRange.push(tap);
			}
		}
		if (tapsInRange.length === 0) return done();
		let counter = tapsInRange.length;
		for (let tap of tapsInRange) {
			this._runTapInterceptors(tap);
			if (tap.type === "promise") {
				const promise = tap.fn(...args2);
				if (!promise || !promise.then) {
					throw new Error(
						"Tap function (tapPromise) did not return promise (returned " +
							promise +
							")"
					);
				}
				promise.then(
					() => {
						counter -= 1;
						if (counter === 0) {
							done();
						}
					},
					(e: Error) => {
						counter = 0;
						error(e);
					}
				);
			} else if (tap.type === "async") {
				tap.fn(...args2, (e: Error) => {
					if (e) {
						counter = 0;
						error(e);
					} else {
						counter -= 1;
						if (counter === 0) {
							done();
						}
					}
				});
			} else {
				let hasError = false;
				try {
					tap.fn(...args2);
				} catch (e) {
					hasError = true;
					counter = 0;
					error(e as Error);
				}
				if (!hasError && --counter === 0) {
					done();
				}
			}
			if (counter <= 0) return;
		}
	}

	tapAsync(
		options: Options<AdditionalOptions>,
		fn: FnWithCallback<T, void>
	): void {
		this._tap("async", options, fn);
	}

	tapPromise(options: Options<AdditionalOptions>, fn: Fn<T, void>): void {
		this._tap("promise", options, fn);
	}
}

export class AsyncSeriesHook<
	T,
	AdditionalOptions = UnsetAdditionalOptions
> extends Hook<T, void, AdditionalOptions> {
	callAsyncStageRange(
		[from, to]: StageRange,
		...args: Append<AsArray<T>, Callback<Error, void>>
	) {
		const args2 = [...args];
		const cb = args2.pop() as Callback<Error, void>;
		if (from === minStage) {
			this._runCallInterceptors(...args2);
		}
		const done = () => {
			this._runDoneInterceptors();
			cb(null);
		};
		const error = (e: Error) => {
			this._runErrorInterceptors(e);
			cb(e);
		};
		const tapsInRange: (FullTap & IfSet<AdditionalOptions>)[] = [];
		for (let tap of this.taps) {
			const stage = tap.stage ?? 0;
			if (from < stage && stage <= to) {
				tapsInRange.push(tap);
			}
		}
		if (tapsInRange.length === 0) return done();
		let index = 0;
		const next = () => {
			const tap = tapsInRange[index];
			this._runTapInterceptors(tap);
			if (tap.type === "promise") {
				const promise = tap.fn(...args2);
				if (!promise || !promise.then) {
					throw new Error(
						"Tap function (tapPromise) did not return promise (returned " +
							promise +
							")"
					);
				}
				promise.then(
					() => {
						index += 1;
						if (index === tapsInRange.length) {
							done();
						} else {
							next();
						}
					},
					(e: Error) => {
						index = tapsInRange.length;
						error(e);
					}
				);
			} else if (tap.type === "async") {
				tap.fn(...args2, (e: Error) => {
					if (e) {
						index = tapsInRange.length;
						error(e);
					} else {
						index += 1;
						if (index === tapsInRange.length) {
							done();
						} else {
							next();
						}
					}
				});
			} else {
				let hasError = false;
				try {
					tap.fn(...args2);
				} catch (e) {
					hasError = true;
					index = tapsInRange.length;
					error(e as Error);
				}
				if (!hasError) {
					index += 1;
					if (index === tapsInRange.length) {
						done();
					} else {
						next();
					}
				}
			}
			if (index === tapsInRange.length) return;
		};
		next();
	}

	tapAsync(
		options: Options<AdditionalOptions>,
		fn: FnWithCallback<T, void>
	): void {
		this._tap("async", options, fn);
	}

	tapPromise(options: Options<AdditionalOptions>, fn: Fn<T, void>): void {
		this._tap("promise", options, fn);
	}
}
