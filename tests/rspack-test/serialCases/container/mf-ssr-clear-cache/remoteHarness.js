function createDeferred() {
	let resolve;
	const promise = new Promise(r => {
		resolve = r;
	});
	return { promise, resolve };
}

export function installRemoteHarness() {
	const harness = {
		version: "v1",
		entryLoads: [],
		gets: [],
		factoryExecutions: [],
		routeExecutions: {
			pageA: 0,
			pageB: 0
		},
		blockNextGet: false,
		pendingGet: undefined,
		pendingGetObserved: undefined,
		setVersion(version) {
			this.version = version;
		},
		blockNextRemoteGet() {
			this.blockNextGet = true;
		},
		waitForPendingGet() {
			if (this.pendingGet) {
				return Promise.resolve();
			}
			return new Promise(resolve => {
				this.pendingGetObserved = resolve;
			});
		},
		resolvePendingGet() {
			const pendingGet = this.pendingGet;
			this.pendingGet = undefined;
			pendingGet.resolve();
		},
		recordRouteExecution(route) {
			this.routeExecutions[route] += 1;
		},
		snapshot() {
			return {
				entryLoads: this.entryLoads.length,
				gets: this.gets.length,
				factoryExecutions: this.factoryExecutions.length,
				routeExecutions: { ...this.routeExecutions }
			};
		},
		loadRemoteEntry() {
			const entryVersion = this.version;
			this.entryLoads.push(entryVersion);
			return {
				init: () => {},
				get: expose => {
					this.gets.push(`${entryVersion}:${expose}`);
					const factory = () => {
						this.factoryExecutions.push(`${entryVersion}:${expose}`);
						return {
							__esModule: true,
							default: `${expose}:${entryVersion}`
						};
					};
					if (this.blockNextGet) {
						this.blockNextGet = false;
						this.pendingGet = createDeferred();
						if (this.pendingGetObserved) {
							this.pendingGetObserved();
							this.pendingGetObserved = undefined;
						}
						return this.pendingGet.promise.then(() => factory);
					}
					return Promise.resolve(factory);
				}
			};
		}
	};
	globalThis.__mfSsrClearCacheHarness = harness;
	return harness;
}
