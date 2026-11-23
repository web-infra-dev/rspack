export default function (source) {
  this.cacheable(false);
  this._compilation.times ??= 0;
  const time = ++this._compilation.times;
  if (time === 2) {
    return source.replaceAll('blue', 'red');
  }
  return source.replaceAll('blue', 'green');
}
