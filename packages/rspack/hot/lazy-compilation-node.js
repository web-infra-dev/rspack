import { createRequire } from 'node:module';

var urlBase = decodeURIComponent(__resourceQuery.slice(1));
var require = createRequire(import.meta.url);

/**
 * @param {{ data: string, onError: (err: Error) => void, active: boolean, module: module }} options options
 * @returns {() => void} function to destroy response
 */
export const activate = function (options) {
  var data = options.data;
  var onError = options.onError;
  var active = options.active;
  var module = options.module;
  /** @type {import("http").IncomingMessage | undefined} */
  var response;
  var disposed = false;

  function errorHandler(err) {
    if (disposed) {
      return;
    }
    err.message =
      'Problem communicating active modules to the server: ' + err.message;
    onError(err);
  }

  var httpModule = urlBase.startsWith('https')
    ? require('https')
    : require('http');
  var request = httpModule.request(
    urlBase,
    {
      method: 'POST',
      agent: false,
      headers: {
        Accept: 'text/event-stream',
        'Content-Type': 'text/plain',
      },
    },
    function (res) {
      response = res;
      response.on('error', errorHandler);
      response.resume();

      if (res.statusCode < 200 || res.statusCode >= 300) {
        onError(
          new Error(
            'Problem communicating active modules to the server: HTTP ' +
              res.statusCode,
          ),
        );
      }

      if (!active && !module.hot) {
        console.log(
          'Hot Module Replacement is not enabled. Waiting for process restart...',
        );
      }
    },
  );

  request.on('error', errorHandler);
  request.write(data);
  request.end();

  return function () {
    disposed = true;
    if (response) {
      response.destroy();
    } else {
      request.destroy();
    }
  };
};
