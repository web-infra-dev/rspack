import { test } from 'uvu';
import assert from 'assert';
import { validate } from '../src/validate';

const schema = require('../src/config/schema.json');

test('context should be string', () => {
  assert.ok(validate(schema, {
    context: 'str'
  }));
  assert.throws(() => {
    validate(schema, {
      context: 123
    })
  })
})

test.run();
