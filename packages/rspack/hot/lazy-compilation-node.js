var urlBase = decodeURIComponent(__resourceQuery.slice(1));
var compiling = new Set();
var errorHandlers = new Set();

/** @type {import("http").ClientRequest | undefined} */
var pendingRequest;
/** @type {boolean} */
var hasPendingUpdate = false;

function sendRequest() {
  if (compiling.size === 0) {
    hasPendingUpdate = false;
    return;
  }

  var modules = Array.from(compiling);
  var data = modules.join('\n');

  var httpModule = urlBase.startsWith('https')
    ? require('https')
    : require('http');

  var request = httpModule.request(
    urlBase,
    {
      method: 'POST',
      agent: false,
      headers: {
        'Content-Type': 'text/plain',
      },
    },
    function (res) {
      pendingRequest = undefined;
      if (res.statusCode < 200 || res.statusCode >= 300) {
        var error = new Error(
          'Problem communicating active modules to the server: HTTP ' +
            res.statusCode,
        );
        errorHandlers.forEach(function (onError) {
          onError(error);
        });
      }
      // Consume response data to free up memory
      res.resume();
      if (hasPendingUpdate) {
        hasPendingUpdate = false;
        sendRequest();
      }
    },
  );

  pendingRequest = request;

  request.on('error', function (err) {
    pendingRequest = undefined;
    var error = new Error(
      'Problem communicating active modules to the server: ' + err.message,
    );
    errorHandlers.forEach(function (onError) {
      onError(error);
    });
  });

  request.write(data);
  request.end();
}

function sendActiveRequest() {
  hasPendingUpdate = true;

  // If no request is pending, start one
  if (!pendingRequest) {
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
