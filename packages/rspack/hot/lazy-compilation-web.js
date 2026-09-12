if (typeof XMLHttpRequest === 'undefined') {
  throw new Error(
    "Environment doesn't support lazy compilation (requires XMLHttpRequest)",
  );
}

var urlBase = decodeURIComponent(__resourceQuery.slice(1));
var activeKeys = new Map();
var errorHandlers = new Set();

/** @type {XMLHttpRequest | undefined} */
var activeXhr;
/** @type {ReturnType<typeof setTimeout> | undefined} */
var reconnectTimer;

var reportError = function reportError(message) {
  var error = new Error(message);
  errorHandlers.forEach(function (onError) {
    onError(error);
  });
};

var sendRequest = function sendRequest() {
  if (activeKeys.size === 0) {
    return;
  }

  var modules = Array.from(activeKeys.keys());
  var data = modules.join('\n');

  var xhr = new XMLHttpRequest();
  activeXhr = xhr;
  xhr.open('POST', urlBase, true);
  // text/plain Content-Type is simple request header
  xhr.setRequestHeader('Content-Type', 'text/plain');
  xhr.setRequestHeader('Accept', 'text/event-stream');

  xhr.onloadend = function () {
    if (activeXhr !== xhr) {
      return;
    }

    activeXhr = undefined;
    if (xhr.status < 200 || xhr.status >= 300) {
      reportError(
        xhr.status
          ? 'Problem communicating active modules to the server: HTTP ' +
              xhr.status
          : 'Problem communicating active modules to the server',
      );
    }

    reconnectTimer = setTimeout(function () {
      reconnectTimer = undefined;
      if (!activeXhr && activeKeys.size) {
        sendRequest();
      }
    }, 1000);
  };

  try {
    xhr.send(data);
  } catch (error) {
    activeXhr = undefined;
    reportError(
      error instanceof Error
        ? 'Problem communicating active modules to the server: ' + error.message
        : 'Problem communicating active modules to the server',
    );
  }
};

var updateRequest = function updateRequest() {
  if (reconnectTimer) {
    clearTimeout(reconnectTimer);
    reconnectTimer = undefined;
  }
  if (activeXhr) {
    var xhr = activeXhr;
    activeXhr = undefined;
    xhr.abort();
  }
  if (activeKeys.size) {
    sendRequest();
  }
};

/**
 * @param {{ data: string, onError: (err: Error) => void, active: boolean, module: module }} options options
 * @returns {() => void} function to destroy response
 */
export const activate = function (options) {
  var data = options.data;
  var onError = options.onError;
  var active = options.active;
  errorHandlers.add(onError);

  var value = activeKeys.get(data) || 0;
  activeKeys.set(data, value + 1);
  if (value === 0) {
    updateRequest();
  }

  if (!active && !import.meta.webpackHot) {
    console.log(
      'Hot Module Replacement is not enabled. Waiting for process restart...',
    );
  }

  return function () {
    errorHandlers.delete(onError);
    // HMR recreates the proxy immediately after disposing it. Delay the
    // decrement so that transition does not flap the server-side connection.
    setTimeout(function () {
      var value = activeKeys.get(data);
      if (value === 1) {
        activeKeys.delete(data);
        updateRequest();
      } else if (value !== undefined) {
        activeKeys.set(data, value - 1);
      }
    }, 1000);
  };
};
