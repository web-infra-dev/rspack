/*
	MIT License http://www.opensource.org/licenses/mit-license.php
	Author Tobias Koppers @sokra
*/
/*globals __resourceQuery */

import { log, formatError } from './log.js';
import { logApplyResult } from './log-apply-result.js';

if (import.meta.webpackHot) {
  /**
   * @param {boolean=} fromUpdate true when called from update
   */
  var checkForUpdate = function checkForUpdate(fromUpdate) {
    import.meta.webpackHot
      .check()
      .then(function (updatedModules) {
        if (!updatedModules) {
          if (fromUpdate) log('info', '[HMR] Update applied.');
          else log('warning', '[HMR] Cannot find update.');
          return;
        }

        return import.meta.webpackHot
          .apply({
            ignoreUnaccepted: true,
            onUnaccepted: function (data) {
              log(
                'warning',
                'Ignored an update to unaccepted module ' +
                  data.chain.join(' -> '),
              );
            },
          })
          .then(function (renewedModules) {
            logApplyResult(updatedModules, renewedModules);

            checkForUpdate(true);
            return null;
          });
      })
      .catch(function (err) {
        var status = import.meta.webpackHot.status();
        if (['abort', 'fail'].indexOf(status) >= 0) {
          log('warning', '[HMR] Cannot apply update.');
          log('warning', '[HMR] ' + formatError(err));
          log('warning', '[HMR] You need to restart the application!');
        } else {
          log('warning', '[HMR] Update failed: ' + (err.stack || err.message));
        }
      });
  };

  process.on(__resourceQuery.slice(1) || 'SIGUSR2', function () {
    if (import.meta.webpackHot.status() !== 'idle') {
      log(
        'warning',
        '[HMR] Got signal but currently in ' +
          import.meta.webpackHot.status() +
          ' state.',
      );
      log('warning', '[HMR] Need to be in idle state to start hot update.');
      return;
    }

    checkForUpdate();
  });
} else {
  throw new Error('[HMR] Hot Module Replacement is disabled.');
}
