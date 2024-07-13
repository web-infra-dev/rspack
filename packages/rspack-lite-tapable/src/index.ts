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
export type FnAsync<T, R> = (
	...args: Append<AsArray<T>, InnerCallback<Error, R>>
) => void;
export type FnPromise<T, R> = (...args: AsArray<T>) => Promise<R>;

declare class UnsetAdditionalOptions {
	_UnsetAdditionalOptions: true;
}
type IfSet<X> = X extends UnsetAdditionalOptions ? {} : X;

export type Callback<E, T> = (error: E | null, result?: T) => void;
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
type ExtractHookArgs<H> = H extends Hook<infer T, any> ? T : never;
type ExtractHookReturn<H> = H extends Hook<any, infer R> ? R : never;
type ExtractHookAdditionalOptions<H> =
	H extends Hook<any, any, infer A> ? A : never;

export interface Hook<
	T = any,
	R = any,
	AdditionalOptions = UnsetAdditionalOptions
> {
	name?: string;
	tap(opt: Options<AdditionalOptions>, fn: Fn<T, R>): void;
	tapAsync(opt: Options<AdditionalOptions>, fn: FnAsync<T, R>): void;
	tapPromise(opt: Options<AdditionalOptions>, fn: FnPromise<T, R>): void;
	intercept(interceptor: HookInterceptor<T, R, AdditionalOptions>): void;
	isUsed(): boolean;
	withOptions(
		opt: TapOptions & IfSet<AdditionalOptions>
	): Hook<T, R, AdditionalOptions>;
	queryStageRange(stageRange: StageRange): QueriedHook<T, R, AdditionalOptions>;
}

export class HookBase<T, R, AdditionalOptions = UnsetAdditionalOptions>
	implements Hook<T, R, AdditionalOptions>
{
	args: ArgumentNames<AsArray<T>>;
	name?: string;
	taps: (FullTap & IfSet<AdditionalOptions>)[];
	interceptors: HookInterceptor<T, R, AdditionalOptions>[];

	constructor(
		args = [] as unknown as ArgumentNames<AsArray<T>>,
		name?: string
	) {
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

	_runResultInterceptors(r: R) {
		for (const interceptor of this.interceptors) {
			if (interceptor.result) {
				interceptor.result(r);
			}
		}
	}

	withOptions(
		options: TapOptions & IfSet<AdditionalOptions>
	): Hook<T, R, AdditionalOptions> {
		const mergeOptions = (opt: any) =>
			Object.assign({}, options, typeof opt === "string" ? { name: opt } : opt);

		return {
			name: this.name,
			tap: (opt: Options<AdditionalOptions>, fn: Fn<T, R>) =>
				this.tap(mergeOptions(opt), fn),
			tapAsync: (opt: Options<AdditionalOptions>, fn: FnAsync<T, R>) =>
				this.tapAsync(mergeOptions(opt), fn),
			tapPromise: (opt: Options<AdditionalOptions>, fn: FnPromise<T, R>) =>
				this.tapPromise(mergeOptions(opt), fn),
			intercept: (interceptor: HookInterceptor<T, R, AdditionalOptions>) =>
				this.intercept(interceptor),
			isUsed: () => this.isUsed(),
			withOptions: (opt: TapOptions & IfSet<AdditionalOptions>) =>
				this.withOptions(mergeOptions(opt)),
			queryStageRange: (stageRange: StageRange) =>
				this.queryStageRange(stageRange)
		};
	}

	isUsed() {
		return this.taps.length > 0 || this.interceptors.length > 0;
	}

	queryStageRange(
		stageRange: StageRange
	): QueriedHook<T, R, AdditionalOptions> {
		return new QueriedHook(stageRange, this);
	}

	callAsyncStageRange(
		queried: QueriedHook<T, R, AdditionalOptions>,
		...args: Append<AsArray<T>, Callback<Error, R>>
	) {
		throw new Error("Hook should implement there own _callAsyncStageRange");
	}

	callAsync(...args: Append<AsArray<T>, Callback<Error, R>>): void {
		return this.callAsyncStageRange(
			this.queryStageRange(allStageRange),
			...args
		);
	}

	promiseStageRange(
		queried: QueriedHook<T, R, AdditionalOptions>,
		...args: AsArray<T>
	): Promise<R> {
		return new Promise((resolve, reject) => {
			this.callAsyncStageRange(
				queried,
				// @ts-expect-error
				...args,
				(e: Error, r: R) => {
					if (e) return reject(e);
					return resolve(r);
				}
			);
		});
	}

	promise(...args: AsArray<T>): Promise<R> {
		return this.promiseStageRange(this.queryStageRange(allStageRange), ...args);
	}

	tap(options: Options<AdditionalOptions>, fn: Fn<T, R>) {
		this._tap("sync", options, fn);
	}

	tapAsync(options: Options<AdditionalOptions>, fn: FnAsync<T, R>) {
		this._tap("async", options, fn);
	}

	tapPromise(options: Options<AdditionalOptions>, fn: FnPromise<T, R>) {
		this._tap("promise", options, fn);
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

	_prepareArgs(args: AsArray<T>): (T | undefined)[] {
		const len = this.args.length;
		if (args.length < len) {
			args.length = len;
			return (args as (T | undefined)[]).fill(undefined, args.length, len);
		}
		if (args.length > len) {
			args.length = len;
			return args;
		}
		return args;
	}
}

export type StageRange = readonly [number, number];
export const minStage = -Infinity;
export const maxStage = Infinity;
const allStageRange = [minStage, maxStage] as const;
const i32MIN = -(2 ** 31);
const i32MAX = 2 ** 31 - 1;
export const safeStage = (stage: number) => {
	if (stage < i32MIN) return i32MIN;
	if (stage > i32MAX) return i32MAX;
	return stage;
};

export class QueriedHook<T, R, AdditionalOptions = UnsetAdditionalOptions> {
	stageRange: StageRange;
	hook: HookBase<T, R, AdditionalOptions>;
	tapsInRange: (FullTap & IfSet<AdditionalOptions>)[];

	constructor(stageRange: StageRange, hook: HookBase<T, R, AdditionalOptions>) {
		const tapsInRange: typeof hook.taps = [];
		const [from, to] = stageRange;
		for (let tap of hook.taps) {
			const stage = tap.stage ?? 0;
			if (from <= stage && stage < to) {
				tapsInRange.push(tap);
			} else if (to === maxStage && stage === maxStage) {
				tapsInRange.push(tap);
			}
		}
		this.stageRange = stageRange;
		this.hook = hook;
		this.tapsInRange = tapsInRange;
	}

	isUsed(): boolean {
		if (this.tapsInRange.length > 0) return true;
		if (
			this.stageRange[0] === minStage &&
			this.hook.interceptors.some(i => i.call)
		)
			return true;
		if (
			this.stageRange[1] === maxStage &&
			this.hook.interceptors.some(i => i.done)
		)
			return true;
		return false;
	}

	call(...args: AsArray<T>): R {
		if (
			typeof (this.hook as SyncHook<T, R, AdditionalOptions>).callStageRange !==
			"function"
		) {
			throw new Error(
				"hook is not a SyncHook, call methods only exists on SyncHook"
			);
		}
		return (this.hook as SyncHook<T, R, AdditionalOptions>).callStageRange(
			this,
			...args
		);
	}

	callAsync(...args: Append<AsArray<T>, Callback<Error, R>>): void {
		return this.hook.callAsyncStageRange(this, ...args);
	}

	promise(...args: AsArray<T>): Promise<R> {
		return this.hook.promiseStageRange(this, ...args);
	}
}

export class SyncHook<
	T,
	R = void,
	AdditionalOptions = UnsetAdditionalOptions
> extends HookBase<T, R, AdditionalOptions> {
	callAsyncStageRange(
		queried: QueriedHook<T, R, AdditionalOptions>,
		...args: Append<AsArray<T>, Callback<Error, R>>
	) {
		const {
			stageRange: [from, to],
			tapsInRange
		} = queried;
		const argsWithoutCb = args.slice(0, args.length - 1) as AsArray<T>;
		const cb = args[args.length - 1] as Callback<Error, R>;
		const args2 = this._prepareArgs(argsWithoutCb);
		if (from === minStage) {
			this._runCallInterceptors(...args2);
		}
		for (let tap of tapsInRange) {
			this._runTapInterceptors(tap);
			try {
				tap.fn(...args2);
			} catch (e) {
				const err = e as Error;
				this._runErrorInterceptors(err);
				return cb(err);
			}
		}
		if (to === maxStage) {
			this._runDoneInterceptors();
			cb(null);
		}
	}

	call(...args: AsArray<T>): R {
		return this.callStageRange(this.queryStageRange(allStageRange), ...args);
	}

	callStageRange(
		queried: QueriedHook<T, R, AdditionalOptions>,
		...args: AsArray<T>
	): R {
		let result, error;
		this.callAsyncStageRange(
			queried,
			// @ts-expect-error
			...args,
			(e: Error, r: R): void => {
				error = e;
				result = r;
			}
		);
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

export class SyncBailHook<
	T,
	R,
	AdditionalOptions = UnsetAdditionalOptions
> extends HookBase<T, R, AdditionalOptions> {
	callAsyncStageRange(
		queried: QueriedHook<T, R, AdditionalOptions>,
		...args: Append<AsArray<T>, Callback<Error, R>>
	) {
		const {
			stageRange: [from, to],
			tapsInRange
		} = queried;
		const argsWithoutCb = args.slice(0, args.length - 1) as AsArray<T>;
		const cb = args[args.length - 1] as Callback<Error, R>;
		const args2 = this._prepareArgs(argsWithoutCb);
		if (from === minStage) {
			this._runCallInterceptors(...args2);
		}
		for (let tap of tapsInRange) {
			this._runTapInterceptors(tap);
			let r = undefined;
			try {
				r = tap.fn(...args2);
			} catch (e) {
				const err = e as Error;
				this._runErrorInterceptors(err);
				return cb(err);
			}
			if (r !== undefined) {
				this._runResultInterceptors(r);
				return cb(null, r);
			}
		}
		if (to === maxStage) {
			this._runDoneInterceptors();
			cb(null);
		}
	}

	call(...args: AsArray<T>): R {
		return this.callStageRange(this.queryStageRange(allStageRange), ...args);
	}

	callStageRange(
		queried: QueriedHook<T, R, AdditionalOptions>,
		...args: AsArray<T>
	): R {
		let result, error;
		this.callAsyncStageRange(
			queried,
			// @ts-expect-error
			...args,
			(e: Error, r: R): void => {
				error = e;
				result = r;
			}
		);
		if (error) {
			throw error;
		}
		return result as R;
	}

	tapAsync(): never {
		throw new Error("tapAsync is not supported on a SyncBailHook");
	}

	tapPromise(): never {
		throw new Error("tapPromise is not supported on a SyncBailHook");
	}
}

export class SyncWaterfallHook<
	T,
	AdditionalOptions = UnsetAdditionalOptions
> extends HookBase<T, AsArray<T>[0], AdditionalOptions> {
	constructor(
		args = [] as unknown as ArgumentNames<AsArray<T>>,
		name?: string
	) {
		if (args.length < 1)
			throw new Error("Waterfall hooks must have at least one argument");
		super(args, name);
	}

	callAsyncStageRange(
		queried: QueriedHook<T, AsArray<T>[0], AdditionalOptions>,
		...args: Append<AsArray<T>, Callback<Error, AsArray<T>[0]>>
	) {
		const {
			stageRange: [from, to],
			tapsInRange
		} = queried;
		const argsWithoutCb = args.slice(0, args.length - 1) as AsArray<T>;
		const cb = args[args.length - 1] as Callback<Error, AsArray<T>[0]>;
		const args2 = this._prepareArgs(argsWithoutCb);
		if (from === minStage) {
			this._runCallInterceptors(...args2);
		}
		for (let tap of tapsInRange) {
			this._runTapInterceptors(tap);
			try {
				const r = tap.fn(...args2);
				if (r !== undefined) {
					args2[0] = r;
				}
			} catch (e) {
				const err = e as Error;
				this._runErrorInterceptors(err);
				return cb(err);
			}
		}
		if (to === maxStage) {
			this._runDoneInterceptors();
			cb(null, args2[0]);
		}
	}

	call(...args: AsArray<T>): AsArray<T>[0] {
		return this.callStageRange(this.queryStageRange(allStageRange), ...args);
	}

	callStageRange(
		queried: QueriedHook<T, AsArray<T>[0], AdditionalOptions>,
		...args: AsArray<T>
	): AsArray<T>[0] {
		let result, error;
		this.callAsyncStageRange(
			queried,
			// @ts-expect-error
			...args,
			(e: Error, r: AsArray<T>[0]): void => {
				error = e;
				result = r;
			}
		);
		if (error) {
			throw error;
		}
		return result as AsArray<T>[0];
	}

	tapAsync(): never {
		throw new Error("tapAsync is not supported on a SyncWaterfallHook");
	}

	tapPromise(): never {
		throw new Error("tapPromise is not supported on a SyncWaterfallHook");
	}
}

export class AsyncParallelHook<
	T,
	AdditionalOptions = UnsetAdditionalOptions
> extends HookBase<T, void, AdditionalOptions> {
	callAsyncStageRange(
		queried: QueriedHook<T, void, AdditionalOptions>,
		...args: Append<AsArray<T>, Callback<Error, void>>
	) {
		const {
			stageRange: [from],
			tapsInRange
		} = queried;
		const argsWithoutCb = args.slice(0, args.length - 1) as AsArray<T>;
		const cb = args[args.length - 1] as Callback<Error, void>;
		const args2 = this._prepareArgs(argsWithoutCb);
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
}

export class AsyncSeriesHook<
	T,
	AdditionalOptions = UnsetAdditionalOptions
> extends HookBase<T, void, AdditionalOptions> {
	callAsyncStageRange(
		queried: QueriedHook<T, void, AdditionalOptions>,
		...args: Append<AsArray<T>, Callback<Error, void>>
	) {
		const {
			stageRange: [from],
			tapsInRange
		} = queried;
		const argsWithoutCb = args.slice(0, args.length - 1) as AsArray<T>;
		const cb = args[args.length - 1] as Callback<Error, void>;
		const args2 = this._prepareArgs(argsWithoutCb);
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
}

export class AsyncSeriesBailHook<
	T,
	R,
	AdditionalOptions = UnsetAdditionalOptions
> extends HookBase<T, R, AdditionalOptions> {
	callAsyncStageRange(
		queried: QueriedHook<T, R, AdditionalOptions>,
		...args: Append<AsArray<T>, Callback<Error, R>>
	) {
		const {
			stageRange: [from],
			tapsInRange
		} = queried;
		const argsWithoutCb = args.slice(0, args.length - 1) as AsArray<T>;
		const cb = args[args.length - 1] as Callback<Error, R>;
		const args2 = this._prepareArgs(argsWithoutCb);
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
		const result = (r: R) => {
			this._runResultInterceptors(r);
			cb(null, r);
		};
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
					(r: R) => {
						index += 1;
						if (r !== undefined) {
							result(r);
						} else if (index === tapsInRange.length) {
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
				tap.fn(...args2, (e: Error, r: R) => {
					if (e) {
						index = tapsInRange.length;
						error(e);
					} else {
						index += 1;
						if (r !== undefined) {
							result(r);
						} else if (index === tapsInRange.length) {
							done();
						} else {
							next();
						}
					}
				});
			} else {
				let hasError = false;
				let r = undefined;
				try {
					r = tap.fn(...args2);
				} catch (e) {
					hasError = true;
					index = tapsInRange.length;
					error(e as Error);
				}
				if (!hasError) {
					index += 1;
					if (r !== undefined) {
						result(r);
					} else if (index === tapsInRange.length) {
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
}

export class AsyncSeriesWaterfallHook<
	T,
	AdditionalOptions = UnsetAdditionalOptions
> extends HookBase<T, AsArray<T>[0], AdditionalOptions> {
	constructor(
		args = [] as unknown as ArgumentNames<AsArray<T>>,
		name?: string
	) {
		if (args.length < 1)
			throw new Error("Waterfall hooks must have at least one argument");
		super(args, name);
	}

	callAsyncStageRange(
		queried: QueriedHook<T, AsArray<T>[0], AdditionalOptions>,
		...args: Append<AsArray<T>, Callback<Error, AsArray<T>[0]>>
	) {
		const {
			stageRange: [from],
			tapsInRange
		} = queried;
		const argsWithoutCb = args.slice(0, args.length - 1) as AsArray<T>;
		const cb = args[args.length - 1] as Callback<Error, AsArray<T>[0]>;
		const args2 = this._prepareArgs(argsWithoutCb);
		if (from === minStage) {
			this._runCallInterceptors(...args2);
		}
		const result = (r: AsArray<T>[0]) => {
			this._runResultInterceptors(r);
			cb(null, r);
		};
		const error = (e: Error) => {
			this._runErrorInterceptors(e);
			cb(e);
		};
		if (tapsInRange.length === 0) return result(args2[0]);
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
					(r: AsArray<T>[0]) => {
						index += 1;
						if (r !== undefined) {
							args2[0] = r;
						}
						if (index === tapsInRange.length) {
							result(args2[0]);
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
				tap.fn(...args2, (e: Error, r: AsArray<T>[0]) => {
					if (e) {
						index = tapsInRange.length;
						error(e);
					} else {
						index += 1;
						if (r !== undefined) {
							args2[0] = r;
						}
						if (index === tapsInRange.length) {
							result(args2[0]);
						} else {
							next();
						}
					}
				});
			} else {
				let hasError = false;
				try {
					const r = tap.fn(...args2);
					if (r !== undefined) {
						args2[0] = r;
					}
				} catch (e) {
					hasError = true;
					index = tapsInRange.length;
					error(e as Error);
				}
				if (!hasError) {
					index += 1;
					if (index === tapsInRange.length) {
						result(args2[0]);
					} else {
						next();
					}
				}
			}
			if (index === tapsInRange.length) return;
		};
		next();
	}
}

const defaultFactory = (key: HookMapKey, hook: unknown) => hook;

export type HookMapKey = any;
export type HookFactory<H> = (key: HookMapKey, hook?: H) => H;
export interface HookMapInterceptor<H> {
	factory?: HookFactory<H>;
}

export class HookMap<H extends Hook> {
	_map: Map<HookMapKey, H> = new Map();
	_factory: HookFactory<H>;
	name?: string;
	_interceptors: HookMapInterceptor<H>[];

	constructor(factory: HookFactory<H>, name?: string) {
		this.name = name;
		this._factory = factory;
		this._interceptors = [];
	}

	get(key: HookMapKey) {
		return this._map.get(key);
	}

	for(key: HookMapKey) {
		const hook = this.get(key);
		if (hook !== undefined) {
			return hook;
		}
		let newHook = this._factory(key);
		const interceptors = this._interceptors;
		for (let i = 0; i < interceptors.length; i++) {
			const factory = interceptors[i].factory;
			if (factory) {
				newHook = factory(key, newHook);
			}
		}
		this._map.set(key, newHook);
		return newHook;
	}

	intercept(interceptor: HookMapInterceptor<H>) {
		this._interceptors.push(
			Object.assign(
				{
					factory: defaultFactory
				},
				interceptor
			)
		);
	}

	isUsed(): boolean {
		for (const key of this._map.keys()) {
			const hook = this.get(key);
			if (hook?.isUsed()) {
				return true;
			}
		}
		return false;
	}

	queryStageRange(stageRange: StageRange): QueriedHookMap<H> {
		return new QueriedHookMap(stageRange, this);
	}
}

export class QueriedHookMap<H extends Hook> {
	stageRange: StageRange;
	hookMap: HookMap<H>;

	constructor(stageRange: StageRange, hookMap: HookMap<H>) {
		this.stageRange = stageRange;
		this.hookMap = hookMap;
	}

	get(key: HookMapKey) {
		return this.hookMap.get(key)?.queryStageRange(this.stageRange);
	}

	for(key: HookMapKey) {
		return this.hookMap.for(key).queryStageRange(this.stageRange);
	}

	isUsed(): boolean {
		for (const key in this.hookMap._map.keys()) {
			if (this.get(key)?.isUsed()) {
				return true;
			}
		}
		return false;
	}
}

export class MultiHook<H extends Hook> {
	hooks: H[];
	name?: string;

	constructor(hooks: H[], name?: string) {
		this.hooks = hooks;
		this.name = name;
	}

	tap(
		options: Options<ExtractHookAdditionalOptions<Hook>>,
		fn: Fn<ExtractHookArgs<Hook>, ExtractHookReturn<Hook>>
	) {
		for (const hook of this.hooks) {
			hook.tap(options, fn);
		}
	}

	tapAsync(
		options: Options<ExtractHookAdditionalOptions<Hook>>,
		fn: FnAsync<ExtractHookArgs<Hook>, ExtractHookReturn<Hook>>
	) {
		for (const hook of this.hooks) {
			hook.tapAsync(options, fn);
		}
	}

	tapPromise(
		options: Options<ExtractHookAdditionalOptions<Hook>>,
		fn: FnPromise<ExtractHookArgs<Hook>, ExtractHookReturn<Hook>>
	) {
		for (const hook of this.hooks) {
			hook.tapPromise(options, fn);
		}
	}

	isUsed() {
		for (const hook of this.hooks) {
			if (hook.isUsed()) return true;
		}
		return false;
	}

	intercept(
		interceptor: HookInterceptor<
			ExtractHookArgs<Hook>,
			ExtractHookReturn<Hook>,
			ExtractHookAdditionalOptions<Hook>
		>
	) {
		for (const hook of this.hooks) {
			hook.intercept(interceptor);
		}
	}

	withOptions(options: TapOptions & IfSet<ExtractHookAdditionalOptions<Hook>>) {
		return new MultiHook(
			this.hooks.map(h => h.withOptions(options)),
			this.name
		);
	}
}
