export default function (content) {
  this.callback(null, content, undefined, { uncloneableResult() {} });
}
