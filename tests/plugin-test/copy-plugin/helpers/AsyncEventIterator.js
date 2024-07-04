/**
 * @template {unknown} T
 * @extends {AsyncIterableIterator<T>}
 * @extends {AsyncIterator<T>}
 */
class AsyncEventIterator {
	constructor() {
		/**
		 * @member {(data: T) => any} resolve
		 */
		this.resolve = data => {};
		/**
		 * @member {(err: unknown) => any} reject
		 */
		this.reject = err => {};
		const self = this;

		this._iter = (async function* createChangesIterator() {
			try {
				while (true) {
					yield await new Promise((resolve, reject) => {
						self.resolve = resolve;
						self.reject = reject;
					});
				}
			} finally {
				self.resolve();
				self.resolve = () => {};
				self.reject = () => {};
			}
		})();
	}
	[Symbol.asyncIterator]() {
		return this;
	}
	next(...args) {
		return this._iter.next(...args);
	}
	return(...args) {
		return this._iter.return(...args);
	}
	throw(...args) {
		return this._iter.throw(...args);
	}
}

module.exports = AsyncEventIterator;
