module.exports = function (code) {
  const time = this.query.match(/time=([0-9]+)/)[1];
  return code.replace('_$TIME_', time);
};
