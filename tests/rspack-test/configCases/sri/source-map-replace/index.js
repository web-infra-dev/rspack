const crypto = require('crypto');
const fs = require('fs');
const path = require('path');

it('keeps integrity and map references correct after streaming RCH replacements', async () => {
  const scripts = [];
  const appendChild = document.head.appendChild.bind(document.head);
  document.head.appendChild = node => {
    if (node.tagName === 'SCRIPT') scripts.push(node);
    return appendChild(node);
  };
  const chunk = await import(/* webpackChunkName: "async" */ './chunk');
  expect(chunk.value).toBe('你好😀');
  expect(scripts).toHaveLength(1);
  const script = scripts[0];
  const filename = path.basename(new URL(script.src).pathname);
  const bytes = fs.readFileSync(path.join(__dirname, filename));
  const expected = ['sha256', 'sha384'].map(algorithm => `${algorithm}-${crypto.createHash(algorithm).update(bytes).digest('base64')}`).join(' ');
  expect(script.integrity).toBe(expected);
  const mapName = bytes.toString().match(/sourceMappingURL=(.+)/)[1].trim();
  const map = JSON.parse(fs.readFileSync(path.join(__dirname, mapName), 'utf8'));
  expect(map.file).toBe(filename);
  expect(map.sourcesContent.join('')).toContain('你好😀');
});
