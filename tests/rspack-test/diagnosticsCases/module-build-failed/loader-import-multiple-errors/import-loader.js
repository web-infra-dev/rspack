const assert = require('assert');

module.exports = async function (content) {
  try {
    await this.importModule('./failed.js');
  } catch (error) {
    assert(error instanceof AggregateError);
    assert.strictEqual(error.errors.length, 2);
    const messages = error.errors.map((error) => error.message);
    assert(messages.some((message) => message.includes('must be initialized')));
    assert(messages.some((message) => message.includes('Expected a semicolon')));
    assert.match(error.message, /must be initialized/);
    assert.match(error.message, /Expected a semicolon/);
    return content;
  }
  throw new Error('Importing the failed module should reject');
};
