export const value = 1;

if (import.meta.hot) {
	import.meta.hot.data.count ||= 0;
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
	import.meta.hot.dispose(data => {
		data.count += 1;
	});
	import.meta.hot.accept();
}
