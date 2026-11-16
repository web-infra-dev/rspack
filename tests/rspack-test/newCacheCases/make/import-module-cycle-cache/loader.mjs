export default function (content) {
  const request = JSON.parse(content);
  const callback = this.async();
  if (request === null) {
    callback(null, JSON.stringify('terminal'));
    return;
  }
  this.importModule(`./loader.mjs!${request}`, {}, (error, value) => {
    callback(null, JSON.stringify(error ? `err: ${error.message}` : `source: ${value}`));
  });
}
