if (typeof fetch !== 'function') {
  throw new Error(
    "Environment doesn't support lazy compilation (requires fetch API)",
  );
}

var urlBase = decodeURIComponent(__resourceQuery.slice(1));
var compiling = new Set();
var errorHandlers = new Set();

/** @type {Promise<void> | undefined} */
var pendingRequestPromise;
/** @type {AbortController | undefined} */
var activeAbortController;
/** @type {boolean} */
var hasPendingUpdate = false;

var sendRequest = function sendRequest() {
  if (compiling.size === 0) {
    pendingRequestPromise = undefined;
    activeAbortController = undefined;
    hasPendingUpdate = false;
    return Promise.resolve();
  }

  var modules = Array.from(compiling);
  activeAbortController = new AbortController();
  var signal = activeAbortController.signal;

  return fetch(urlBase, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Accept: 'text/event-stream',
    },
    body: JSON.stringify(modules),
    signal: signal,
  })
    .then(function (response) {
      if (!response.ok) {
        var error = new Error(
          'Problem communicating active modules to the server: HTTP ' +
            response.status,
        );
        errorHandlers.forEach(function (onError) {
          onError(error);
        });
      }
      // The response is kept alive for server-side event streaming,
      // but we don't need to process events for lazy compilation.
    })
    .catch(function (err) {
      if (err.name === 'AbortError') {
        // Request was aborted, which is expected when deactivating
        return;
      }
      errorHandlers.forEach(function (onError) {
        onError(err);
      });
    });
};

var updateEventSource = function updateEventSource() {
  hasPendingUpdate = true;

  // If no request is pending, start one
  if (!pendingRequestPromise) {
    pendingRequestPromise = sendRequest().finally(function () {
      // After the request completes, check if there are pending updates
      pendingRequestPromise = undefined;
    });
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
      'Hot Module Replacement is not enabled. Waiting for process restart...',
    );
  }

  return function () {
    errorHandlers.delete(onError);
    compiling.delete(data);
    updateEventSource();
  };
};
