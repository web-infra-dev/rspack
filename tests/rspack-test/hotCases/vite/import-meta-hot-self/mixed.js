export const value = 1;

globalThis.__mixedSelfEvaluations =
	(globalThis.__mixedSelfEvaluations || 0) + 1;

if (module.hot) {
	module.hot.accept(error => {
		globalThis.__mixedSelfWebpackError = error.message;
	});
}

if (import.meta.hot) {
	import.meta.hot.accept(mod => {
		globalThis.__mixedSelfAccepted = {
			value: mod.value,
			calls: (globalThis.__mixedSelfAccepted?.calls || 0) + 1
		};
	});
}

---

export const value = 2;

globalThis.__mixedSelfEvaluations += 1;

if (module.hot) {
	module.hot.accept(error => {
		globalThis.__mixedSelfWebpackError = error.message;
	});
}

if (import.meta.hot) {
	import.meta.hot.accept(mod => {
		globalThis.__mixedSelfAccepted = {
			value: mod.value,
			calls: (globalThis.__mixedSelfAccepted?.calls || 0) + 1
		};
	});
}

throw new Error("mixed self failure");

---

export const value = 3;

globalThis.__mixedSelfEvaluations += 1;

if (module.hot) {
	module.hot.accept(error => {
		globalThis.__mixedSelfWebpackError = error.message;
	});
}

if (import.meta.hot) {
	import.meta.hot.accept(mod => {
		globalThis.__mixedSelfAccepted = {
			value: mod.value,
			calls: (globalThis.__mixedSelfAccepted?.calls || 0) + 1
		};
	});
}
