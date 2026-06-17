function createDeferred() {
	let resolve;
	let reject;
	const promise = new Promise((r, j) => {
		resolve = r;
		reject = j;
	});
	return { promise, resolve, reject };
}

export function createRemoteServer() {
	const remoteServer = {
		version: "v1",
		remoteEntryLoads: [],
		remoteGets: [],
		factoryExecutions: [],
		setVersion(version) {
			this.version = version;
		},
		snapshot() {
			return {
				remoteEntryLoads: this.remoteEntryLoads.length,
				remoteGets: this.remoteGets.length,
				factoryExecutions: this.factoryExecutions.length
			};
		},
		loadRemoteEntry() {
			const entryVersion = this.version;
			this.remoteEntryLoads.push(entryVersion);
			return {
				init: () => {},
				get: expose => {
					this.remoteGets.push(`${entryVersion}:${expose}`);
					return Promise.resolve(() => {
						this.factoryExecutions.push(`${entryVersion}:${expose}`);
						return {
							__esModule: true,
							default: `${expose}:${entryVersion}`
						};
					});
				}
			};
		}
	};
	globalThis.__mfSsrClearCacheAsyncChunkRemoteServer = remoteServer;
	return remoteServer;
}

export { createDeferred };
