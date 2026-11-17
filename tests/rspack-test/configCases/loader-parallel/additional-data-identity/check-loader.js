module.exports = function (content, map, data) {
  const original = require('./value');
  if (data !== original) throw new Error('main object identity was lost');
  return `module.exports = ${JSON.stringify({ count: data.count, buffer: data.buffer.toString(), value: data.map.get('value') })}`;
};
