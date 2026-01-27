if (typeof XMLHttpRequest === 'undefined') {
  throw new Error(
    "Environment doesn't support lazy compilation (requires XMLHttpRequest)",
  );
}

var urlBase = decodeURIComponent(__resourceQuery.slice(1));
var compiling = new Set();
var errorHandlers = new Set();

/** @type {XMLHttpRequest | undefined} */
var pendingXhr;
/** @type {boolean} */
var hasPendingUpdate = false;

var sendRequest = function sendRequest() {
  if (compiling.size === 0) {
    hasPendingUpdate = false;
    return;
  }

  var modules = Array.from(compiling);
  var data = modules.join('\n');

  var xhr = new XMLHttpRequest();
  pendingXhr = xhr;
  xhr.open('POST', urlBase, true);
  // text/plain Content-Type is simple request header
  xhr.setRequestHeader('Content-Type', 'text/plain');

  xhr.onreadystatechange = function () {
    if (xhr.readyState === 4) {
      pendingXhr = undefined;
      if (xhr.status < 200 || xhr.status >= 300) {
        var error = new Error(
          'Problem communicating active modules to the server: HTTP ' +
            xhr.status,
        );
        errorHandlers.forEach(function (onError) {
          onError(error);
        });
      }
      if (hasPendingUpdate) {
        hasPendingUpdate = false;
        sendRequest();
      }
    }
  };

  xhr.onerror = function () {
    pendingXhr = undefined;
    var error = new Error('Problem communicating active modules to the server');
    errorHandlers.forEach(function (onError) {
      onError(error);
    });
  };

  xhr.send(data);
};

function sendActiveRequest() {
  hasPendingUpdate = true;

  // If no request is pending, start one
  if (!pendingXhr) {
    hasPendingUpdate = false;
    sendRequest();
  }
}

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
    sendActiveRequest();
  }

  if (!active && !module.hot) {
    console.log(
      'Hot Module Replacement is not enabled. Waiting for process restart...',
    );
  }

  return function () {
    errorHandlers.delete(onError);
    compiling.delete(data);
    sendActiveRequest();
  };
};
