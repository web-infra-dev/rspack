/*
	MIT License http://www.opensource.org/licenses/mit-license.php
	Author Tobias Koppers @sokra
*/
/*globals __webpack_hash__ */

import { log, formatError } from './log.js';
import { emitter as hotEmitter } from './emitter.js';
import { logApplyResult } from './log-apply-result.js';

if (import.meta.webpackHot) {
  /** @type {undefined|string} */
  var lastHash;
  var upToDate = function upToDate() {
    return /** @type {string} */ (lastHash).indexOf(__webpack_hash__) >= 0;
  };
  var check = function check() {
    import.meta.webpackHot
      .check()
      .then(function (updatedModules) {
        if (!updatedModules) {
          log('warning', '[HMR] Cannot find update. Need to do a full reload!');
          log(
            'warning',
            '[HMR] (Probably because of restarting the webpack-dev-server)',
          );
          return;
        }

        return import.meta.webpackHot
          .apply({
            ignoreUnaccepted: true,
            ignoreDeclined: true,
            ignoreErrored: true,
            onUnaccepted: function (data) {
              log(
                'warning',
                'Ignored an update to unaccepted module ' +
                  data.chain.join(' -> '),
              );
            },
            onDeclined: function (data) {
              log(
                'warning',
                'Ignored an update to declined module ' +
                  data.chain.join(' -> '),
              );
            },
            onErrored: function (data) {
              log('error', data.error);
              log(
                'warning',
                'Ignored an error while updating module ' +
                  data.moduleId +
                  ' (' +
                  data.type +
                  ')',
              );
            },
          })
          .then(function (renewedModules) {
            if (!upToDate()) {
              check();
            }

            logApplyResult(updatedModules, renewedModules);

            if (upToDate()) {
              log('info', '[HMR] App is up to date.');
            }
          });
      })
      .catch(function (err) {
        var status = import.meta.webpackHot.status();
        if (['abort', 'fail'].indexOf(status) >= 0) {
          log(
            'warning',
            '[HMR] Cannot check for update. Need to do a full reload!',
          );
          log('warning', '[HMR] ' + formatError(err));
        } else {
          log('warning', '[HMR] Update check failed: ' + formatError(err));
        }
      });
  };
  hotEmitter.on('webpackHotUpdate', function (currentHash) {
    lastHash = currentHash;
    if (!upToDate()) {
      var status = import.meta.webpackHot.status();
      if (status === 'idle') {
        log('info', '[HMR] Checking for updates on the server...');
        check();
      } else if (['abort', 'fail'].indexOf(status) >= 0) {
        log(
          'warning',
          '[HMR] Cannot apply update as a previous update ' +
            status +
            'ed. Need to do a full reload!',
        );
      }
    }
  });
  log('info', '[HMR] Waiting for update signal from WDS...');
} else {
  throw new Error('[HMR] Hot Module Replacement is disabled.');
}
