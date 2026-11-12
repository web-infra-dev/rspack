const path = require('node:path');

module.exports = async function (source) {
  if (this.recoveryState.loaderError) {
    throw new Error('expected module build error');
  }
  if (this.recoveryState.fail && this.recoveryState.executorError) {
    await this.importModule(path.join(this.context, 'executed.js'));
  }
  return source;
};
