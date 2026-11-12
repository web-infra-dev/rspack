const path = require('node:path');
const { setTimeout: delay } = require('node:timers/promises');

module.exports = async function (source) {
  this.cacheable(false);
  const state = this.cleanupState;
  const name = path.basename(this.resourcePath);
  if (name === 'outer.js') {
    const result = await this.importModule(path.join(__dirname, 'entry.js'));
    return `export default ${JSON.stringify(result.default)};`;
  }
  if (!state.failing) return source;
  if (name === 'a.js') {
    await (state.nestedBeforeFailure ? state.nestedStarted : state.bStarted)
      .promise;
    return source;
  }

  state.bStarted.resolve();
  if (!state.nestedBeforeFailure) {
    await state.failureObserved.promise;
    await delay(20);
  }
  try {
    const result = await this.importModule(
      `!!${path.join(__dirname, 'payload-loader.cjs')}!${path.join(__dirname, 'payload.js')}`,
    );
    state.imported = result.default;
  } catch (error) {
    state.importError = error;
  }
  state.bFinished = true;
  return source;
};
