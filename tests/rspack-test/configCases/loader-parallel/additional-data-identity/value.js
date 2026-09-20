class AdditionalData {
  count = 0;
  buffer = Buffer.from('native-handle');
  map = new Map([['value', 42]]);
  increment() { return ++this.count; }
}
module.exports = new AdditionalData();
