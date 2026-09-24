export default function (content) {
  let reads = 0;
  this._module.buildInfo.payload = {
    get value() {
      return ++reads;
    },
  };
  return content;
}
