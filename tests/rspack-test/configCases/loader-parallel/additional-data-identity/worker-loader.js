module.exports = function (content, map, data) {
  if (!this.parallel) throw new Error('expected worker execution');
  data.increment();
  if (!Buffer.isBuffer(data.buffer)) throw new Error('buffer must stay a Buffer');
  this.callback(null, content, map, data);
};
