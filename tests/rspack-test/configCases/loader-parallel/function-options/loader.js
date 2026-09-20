const { isMainThread } = require('node:worker_threads');

module.exports = function () {
  const options = this.getOptions();
  let thrown;
  try {
    options.fail();
  } catch (error) {
    thrown = error.message;
  }
  return `module.exports = ${JSON.stringify({
    value: options.transform.call(options, 'value'),
    nested: options.invoke('value', value => `worker:${value}`),
    thrown,
    map: options.map.get('key'),
    typed: [...options.typed],
    url: options.url.href,
    custom: this.customTransform(this.customValue),
    hook: this.hookTransform(this.hookValue),
    hookMainThread: this.hookMainThread,
    worker: !isMainThread,
  })}`;
};
