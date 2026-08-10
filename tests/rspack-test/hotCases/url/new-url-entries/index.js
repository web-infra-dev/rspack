import { cssUrl, jsUrl } from './urls';

const fs = require('fs');
const path = require('path');

const readOutput = (url) =>
  fs.readFileSync(
    path.join(__dirname, new URL(url).pathname.split('/').pop()),
    'utf-8',
  );

it('should update new URL CSS and JS entries', async () => {
  const firstCssUrl = cssUrl.href;
  const firstJsUrl = jsUrl.href;

  expect(readOutput(firstCssUrl)).toContain('color: red');
  expect(readOutput(firstJsUrl)).toContain('script-v1');

  await NEXT_HMR();

  expect(cssUrl.href).toBe(firstCssUrl);
  expect(jsUrl.href).toBe(firstJsUrl);

  expect(readOutput(cssUrl.href)).toContain('color: blue');
  expect(readOutput(jsUrl.href)).toContain('script-v2');
});

module.hot.accept('./urls');
