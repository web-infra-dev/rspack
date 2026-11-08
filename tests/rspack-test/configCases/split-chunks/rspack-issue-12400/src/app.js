import _ from 'underscore';

it('should remove empty chunks', async () => {
  const [asyncBar, asyncFoo] = await Promise.all([
    import(/* webpackChunkName: "async-bar" */ './async-bar.js').then(m => m.default),
    import(/* webpackChunkName: "async-foo" */ './async-foo.js').then(m => m.default),
  ]);

  expect(asyncBar).toBe('async-bar');
  expect(asyncFoo).toBe('foo');

  const path = require('path')
  const fs = require('fs')

  const summary = fs.readFileSync(path.join(__dirname, 'chunks-summary.txt'), 'utf-8');
  expect(summary).not.toContain('app~app2');
})
