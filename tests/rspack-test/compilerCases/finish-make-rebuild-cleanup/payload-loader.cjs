const { setTimeout: delay } = require('node:timers/promises');

module.exports = async function (source) {
  this.cacheable(false);
  const state = this.cleanupState;
  state.nestedStarted.resolve();
  await state.failureObserved.promise;
  await delay(20);
  this.emitFile('failed-import.txt', 'from the failed compilation');
  state.payloadFinished = true;
  return source;
};
