if (typeof EventSource !== "function") {
	throw new Error(
		"Environment doesn't support lazy compilation (requires EventSource)"
	);
}

var urlBase = decodeURIComponent(__resourceQuery.slice(1));
/** @type {EventSource | undefined} */
var activeEventSource;
var compiling = new Set();
var errorHandlers = new Set();

var updateEventSource = function updateEventSource() {
	if (activeEventSource) activeEventSource.close();
	if (compiling.size) {
		activeEventSource = new EventSource(
			urlBase +
				Array.from(compiling, function (module) {
					return encodeURIComponent(module);
				}).join("@")
		);
		/**
		 * @this {EventSource}
		 * @param {Event & { message?: string, filename?: string, lineno?: number, colno?: number, error?: Error }} event event
		 */
		activeEventSource.onerror = function (event) {
			errorHandlers.forEach(function (onError) {
				onError(
					new Error(
						"Problem communicating active modules to the server" +
							(event.message ? `: ${event.message} ` : "") +
							(event.filename ? `: ${event.filename} ` : "") +
							(event.lineno ? `: ${event.lineno} ` : "") +
							(event.colno ? `: ${event.colno} ` : "") +
							(event.error ? `: ${event.error}` : "")
					)
				);
			});
		};
	} else {
		activeEventSource = undefined;
	}
};

/**
 * @param {{ data: string, onError: (err: Error) => void, active: boolean, module: module }} options options
 * @returns {() => void} function to destroy response
 */
exports.activate = function (options) {
	var data = options.data;
	var onError = options.onError;
	var active = options.active;
	var module = options.module;
	errorHandlers.add(onError);

	if (!compiling.has(data)) {
		compiling.add(data);
		updateEventSource();
	}

	if (!active && !module.hot) {
		console.log(
			"Hot Module Replacement is not enabled. Waiting for process restart..."
		);
	}

	return function () {
		errorHandlers.delete(onError);
		compiling.delete(data);
		updateEventSource();
	};
};
