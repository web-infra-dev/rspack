declare module "neo-async" {
	interface QueueObject<T, E> {
		push(item: T): void;
		drain: () => void;
		error: (err: E) => void;
	}

	export interface Dictionary<T> {
		[key: string]: T;
	}
	export type IterableCollection<T> = T[] | Iterable<T> | Dictionary<T>;

	export interface ErrorCallback<T> {
		(err?: T): void;
	}
	export interface AsyncBooleanResultCallback<E> {
		(err?: E, truthValue?: boolean): void;
	}
	export interface AsyncResultCallback<T, E> {
		(err?: E, result?: T): void;
	}
	export interface AsyncResultArrayCallback<T, E> {
		(err?: E, results?: Array<T | undefined>): void;
	}
	export interface AsyncResultObjectCallback<T, E> {
		(err: E | undefined, results: Dictionary<T | undefined>): void;
	}

	export interface AsyncFunction<T, E> {
		(callback: (err?: E, result?: T) => void): void;
	}
	export interface AsyncFunctionEx<T, E> {
		(callback: (err?: E, ...results: T[]) => void): void;
	}
	export interface AsyncIterator<T, E> {
		(item: T, callback: ErrorCallback<E>): void;
	}
	export interface AsyncForEachOfIterator<T, E> {
		(item: T, key: number | string, callback: ErrorCallback<E>): void;
	}
	export interface AsyncResultIterator<T, R, E> {
		(item: T, callback: AsyncResultCallback<R, E>): void;
	}
	export interface AsyncMemoIterator<T, R, E> {
		(memo: R | undefined, item: T, callback: AsyncResultCallback<R, E>): void;
	}
	export interface AsyncBooleanIterator<T, E> {
		(item: T, callback: AsyncBooleanResultCallback<E>): void;
	}

	export interface AsyncWorker<T, E> {
		(task: T, callback: ErrorCallback<E>): void;
	}
	export interface AsyncVoidFunction<E> {
		(callback: ErrorCallback<E>): void;
	}

	export type AsyncAutoTasks<R extends Dictionary<any>, E> = {
		[K in keyof R]: AsyncAutoTask<R[K], R, E>;
	};
	export type AsyncAutoTask<R1, R extends Dictionary<any>, E> =
		| AsyncAutoTaskFunctionWithoutDependencies<R1, E>
		| (keyof R | AsyncAutoTaskFunction<R1, R, E>)[];
	export interface AsyncAutoTaskFunctionWithoutDependencies<R1, E> {
		(cb: AsyncResultCallback<R1, E> | ErrorCallback<E>): void;
	}
	export interface AsyncAutoTaskFunction<R1, R extends Dictionary<any>, E> {
		(results: R, cb: AsyncResultCallback<R1, E> | ErrorCallback<E>): void;
	}

	export function each<T, E>(
		arr: IterableCollection<T>,
		iterator: AsyncIterator<T, E>,
		callback?: ErrorCallback<E>
	): void;

	export function eachLimit<T, E>(
		arr: IterableCollection<T>,
		limit: number,
		iterator: AsyncIterator<T, E>,
		callback?: ErrorCallback<E>
	): void;

	export function map<T, R, E>(
		arr: T[] | IterableIterator<T>,
		iterator: AsyncResultIterator<T, R, E>,
		callback?: AsyncResultArrayCallback<R, E>
	): void;
	export function map<T, R, E>(
		arr: Dictionary<T>,
		iterator: AsyncResultIterator<T, R, E>,
		callback?: AsyncResultArrayCallback<R, E>
	): void;

	export function parallel<T, E>(
		tasks: Array<AsyncFunction<T, E>>,
		callback?: AsyncResultArrayCallback<T, E>
	): void;
	export function parallel<T, E>(
		tasks: Dictionary<AsyncFunction<T, E>>,
		callback?: AsyncResultObjectCallback<T, E>
	): void;

	export function queue<T, E>(
		worker: AsyncIterator<T, E>,
		concurrency?: number
	): QueueObject<T, E>;

	export const forEach: typeof each;
	export const forEachLimit: typeof eachLimit;
}

declare module "webpack-sources" {
	export type MapOptions = { columns?: boolean; module?: boolean };

	export type RawSourceMap = {
		version: number;
		sources: string[];
		names: string[];
		sourceRoot?: string;
		sourcesContent?: string[];
		mappings: string;
		file: string;
	};

	export abstract class Source {
		size(): number;

		map(options?: MapOptions): RawSourceMap | null;

		sourceAndMap(options?: MapOptions): {
			source: string | Buffer;
			map: Object;
		};

		updateHash(hash: Object): void;

		source(): string | Buffer;

		buffer(): Buffer;
	}

	export class RawSource extends Source {
		constructor(source: string | Buffer, convertToString?: boolean);

		isBuffer(): boolean;
	}

	export class OriginalSource extends Source {
		constructor(source: string | Buffer, name: string);

		getName(): string;
	}

	export class ReplaceSource extends Source {
		constructor(source: Source, name?: string);

		replace(start: number, end: number, newValue: string, name?: string): void;
		insert(pos: number, newValue: string, name?: string): void;

		getName(): string;
		original(): string;
		getReplacements(): {
			start: number;
			end: number;
			content: string;
			insertIndex: number;
			name: string;
		}[];
	}

	export class SourceMapSource extends Source {
		constructor(
			source: string | Buffer,
			name: string,
			sourceMap: Object | string | Buffer,
			originalSource?: string | Buffer,
			innerSourceMap?: Object | string | Buffer,
			removeOriginalSource?: boolean
		);

		getArgsAsBuffers(): [
			Buffer,
			string,
			Buffer,
			Buffer | undefined,
			Buffer | undefined,
			boolean
		];
	}

	export class ConcatSource extends Source {
		constructor(...args: (string | Source)[]);

		getChildren(): Source[];

		add(item: string | Source): void;
		addAllSkipOptimizing(items: Source[]): void;
	}

	export class PrefixSource extends Source {
		constructor(prefix: string, source: string | Source);

		original(): Source;
		getPrefix(): string;
	}

	export class CachedSource extends Source {
		constructor(source: Source);
		constructor(source: Source | (() => Source), cachedData?: any);

		original(): Source;
		originalLazy(): Source | (() => Source);
		getCachedData(): any;
	}

	export class SizeOnlySource extends Source {
		constructor(size: number);
	}

	interface SourceLike {
		source(): string | Buffer;
	}

	export class CompatSource extends Source {
		constructor(sourceLike: SourceLike);

		static from(sourceLike: SourceLike): Source;
	}
}
