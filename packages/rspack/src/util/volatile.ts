class MicrotaskQueue {
	#callbacks: (() => void)[] = [];

	queue(callback: () => void) {
		if (this.#callbacks.length === 0) {
			queueMicrotask(() => {
				const callbacks = this.#callbacks;
				this.#callbacks = [];
				for (const cb of callbacks) {
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
	#setted = false;
	#value: V | undefined = undefined;

	get(): V | undefined {
		return this.#value;
	}

	set(value: V) {
		if (!this.#setted) {
			GLOBAL_MICROTASK_QUEUE.queue(() => {
				this.#value = undefined;
				this.#setted = false;
			});
		}
		this.#value = value;
		this.#setted = true;
	}

	has() {
		return this.#setted;
	}
}
