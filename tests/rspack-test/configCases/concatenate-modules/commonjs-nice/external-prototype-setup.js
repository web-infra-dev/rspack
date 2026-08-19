const exportName = "__rspack_cjs_external_setter__";
const observationName = "__rspack_cjs_external_setter_seen__";
const originalDescriptor = Object.getOwnPropertyDescriptor(
	Object.prototype,
	exportName
);

Object.defineProperty(Object.prototype, exportName, {
	configurable: true,
	get() {
		return globalThis[observationName];
	},
	set(value) {
		globalThis[observationName] = value;
	}
});

export function restoreExternalPrototypeSetter() {
	if (originalDescriptor) {
		Object.defineProperty(Object.prototype, exportName, originalDescriptor);
	} else {
		delete Object.prototype[exportName];
	}
	delete globalThis[observationName];
}
