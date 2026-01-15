/*
	MIT License http://www.opensource.org/licenses/mit-license.php
	Author Tobias Koppers @sokra
*/
/*globals __resourceQuery */

import { log, formatError } from './log.js';
import { logApplyResult } from './log-apply-result.js';

if (import.meta.webpackHot) {
  var hotPollInterval = +__resourceQuery.slice(1) || 10 * 60 * 1000;

  /**
   * @param {boolean=} fromUpdate true when called from update
   */
  var checkForUpdate = function checkForUpdate(fromUpdate) {
    if (import.meta.webpackHot.status() === 'idle') {
      import.meta.webpackHot
        .check(true)
        .then(function (updatedModules) {
          if (!updatedModules) {
            if (fromUpdate) log('info', '[HMR] Update applied.');
            return;
          }
          logApplyResult(updatedModules, updatedModules);
          checkForUpdate(true);
        })
        .catch(function (err) {
          var status = import.meta.webpackHot.status();
          if (['abort', 'fail'].indexOf(status) >= 0) {
            log('warning', '[HMR] Cannot apply update.');
            log('warning', '[HMR] ' + formatError(err));
            log('warning', '[HMR] You need to restart the application!');
          } else {
            log('warning', '[HMR] Update failed: ' + formatError(err));
          }
        });
    }
  };
  setInterval(checkForUpdate, hotPollInterval);
} else {
  throw new Error('[HMR] Hot Module Replacement is disabled.');
}
