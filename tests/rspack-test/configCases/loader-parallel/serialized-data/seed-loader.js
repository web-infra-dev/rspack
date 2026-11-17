module.exports = function (content) {
  const shared = { value: 7 };
  const data = {
    count: 0,
    shared,
    alias: shared,
    buffer: Buffer.from('additional data'),
    map: new Map([['shared', shared]]),
    set: new Set([shared]),
    typed: new Uint16Array([256, 1024]),
    bigint: 12345678901234567890n,
    missing: undefined,
  };
  data.self = data;
  this.callback(null, content, null, data);
};
