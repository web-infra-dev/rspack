class MicrotaskQueue {
	#callbacks: (() => void)[] = [];

	queue(callback: () => void) {
		if (this.#callbacks.length === 0) {
			queueMicrotask(() => {
				for (const cb of this.#callbacks) {
					cb();
				}
			});
		}
		this.#callbacks.push(callback);
	}
}

const GLOBAL_MICROTASK_QUEUE = new MicrotaskQueue();

export class VolatileMap<K, V> {
	#map = new Map<K, V>();

	get(key: K): V | undefined {
		return this.#map.get(key);
	}

	set(key: K, value: V) {
		if (this.#map.size === 0) {
			GLOBAL_MICROTASK_QUEUE.queue(() => {
				this.#map.clear();
			});
		}
		this.#map.set(key, value);
	}

	has(key: K): boolean {
		return this.#map.has(key);
	}
}

export class VolatileValue<V> {
	#value: V | undefined = undefined;

	get(): V | undefined {
		return this.#value;
	}

	set(value: V) {
		if (this.#value === undefined) {
			GLOBAL_MICROTASK_QUEUE.queue(() => {
				this.#value = undefined;
			});
		}
		this.#value = value;
	}
}
