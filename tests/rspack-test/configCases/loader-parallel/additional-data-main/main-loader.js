const { isMainThread } = require("node:worker_threads");

module.exports = function (content, sourceMap, additionalData) {
  this.callback(
    null,
    `module.exports = ${JSON.stringify({
      main: isMainThread,
      buffer:
        Buffer.isBuffer(additionalData.buffer) &&
        additionalData.buffer.toString(),
      map:
        additionalData.map instanceof Map && additionalData.map.get("key"),
    })}`,
  );
};
