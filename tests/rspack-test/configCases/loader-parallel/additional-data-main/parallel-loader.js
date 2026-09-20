module.exports = function (content) {
  this.callback(null, content, null, {
    buffer: Buffer.from("worker"),
    map: new Map([["key", "value"]]),
  });
};
