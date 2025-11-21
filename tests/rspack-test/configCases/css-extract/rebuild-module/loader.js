let time = 0;
module.exports = function (source) {
  this.cacheable(false);
  time++;
  if (time === 2) {
    return source.replaceAll('blue', 'red');
  }
  return source.replaceAll('blue', 'green');
}
