/*
	MIT License http://www.opensource.org/licenses/mit-license.php
	Author Tobias Koppers @sokra
*/
/* globals __webpack_hash__ */

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
      .check(true)
      .then(function (updatedModules) {
        if (!updatedModules) {
          log(
            'warning',
            '[HMR] Cannot find update. ' +
              (typeof window !== 'undefined'
                ? 'Need to do a full reload!'
                : 'Please reload manually!'),
          );
          log(
            'warning',
            '[HMR] (Probably because of restarting the webpack-dev-server)',
          );
          if (typeof window !== 'undefined') {
            window.location.reload();
          }
          return;
        }

        if (!upToDate()) {
          check();
        }

        logApplyResult(updatedModules, updatedModules);

        if (upToDate()) {
          log('info', '[HMR] App is up to date.');
        }
      })
      .catch(function (err) {
        var status = import.meta.webpackHot.status();
        if (['abort', 'fail'].indexOf(status) >= 0) {
          log(
            'warning',
            '[HMR] Cannot apply update. ' +
              (typeof window !== 'undefined'
                ? 'Need to do a full reload!'
                : 'Please reload manually!'),
          );
          log('warning', '[HMR] ' + formatError(err));
          if (typeof window !== 'undefined') {
            window.location.reload();
          }
        } else {
          log('warning', '[HMR] Update failed: ' + formatError(err));
        }
      });
  };
  hotEmitter.on('webpackHotUpdate', function (currentHash) {
    lastHash = currentHash;
    if (!upToDate() && import.meta.webpackHot.status() === 'idle') {
      log('info', '[HMR] Checking for updates on the server...');
      check();
    }
  });
  log('info', '[HMR] Waiting for update signal from WDS...');
} else {
  throw new Error('[HMR] Hot Module Replacement is disabled.');
}
