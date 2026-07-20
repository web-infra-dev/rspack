export const value = 1;

if (module.hot) {
	module.hot.dispose(data => {
		data.webpackValue = "webpack";
	});
}

if (import.meta.hot) {
	import.meta.hot.data.count ||= 0;
	globalThis.__importMetaHotData = import.meta.hot.data;
	globalThis.__importMetaHotInitial = {
		value,
		count: import.meta.hot.data.count
	};
	import.meta.hot.dispose(data => {
		data.count += 1;
	});
	import.meta.hot.accept(mod => {
		globalThis.__importMetaHotAccepted = {
			value: mod.value,
			count: import.meta.hot.data.count
		};
	});
}

---

export const value = 2;

if (import.meta.hot) {
	globalThis.__importMetaHotDataIdentity =
		import.meta.hot.data === globalThis.__importMetaHotData;
	globalThis.__importMetaHotDataStoredInWebpackData =
		module.hot &&
		module.hot.data &&
		Object.getOwnPropertyNames(module.hot.data).some(
			key => module.hot.data[key] === import.meta.hot.data
		);
	globalThis.__importMetaHotWebpackDataKeys = module.hot
		? Object.keys(module.hot.data)
		: [];
	import.meta.hot.dispose(data => {
		data.count += 1;
	});
	import.meta.hot.accept();
}
