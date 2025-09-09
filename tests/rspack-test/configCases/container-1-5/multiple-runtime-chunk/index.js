const fs = require('fs');
const path = require('path');

it('should load the component from container', () => {
  return import('./App').then(({ default: App }) => {
    const rendered = App();
    expect(rendered).toContain('App rendered');
    return import('./upgrade-react').then(({ default: upgrade }) => {
      upgrade();
      const rendered = App();
      expect(rendered).toContain('App rendered');
    });
  });
});

it('should have correct __webpack_require__.x distribution in entry chunks', () => {
  const distPath = __dirname;
  const isESM = distPath.includes('/module') || distPath.includes('\\module');
  const ext = isESM ? '.mjs' : '.js';
  
  const countInFile = (filename, pattern) => {
    const filePath = path.join(distPath, filename.replace('.js', ext));
    const content = fs.readFileSync(filePath, 'utf-8');
    const matches = content.match(new RegExp(pattern, 'g'));
    return matches ? matches.length : 0;
  };

  expect(countInFile('another.js', '__webpack_require__\\.x\\(\\)')).toBe(1);
  expect(countInFile('main.js', '__webpack_require__\\.x\\(\\)')).toBe(1);
});

it('should have correct __webpack_require__.x distribution in runtime chunks', () => {
  const distPath = __dirname;
  const isESM = distPath.includes('/module') || distPath.includes('\\module');
  const ext = isESM ? '.mjs' : '.js';
  
  const countInFile = (filename, pattern) => {
    const filePath = path.join(distPath, filename.replace('.js', ext));
    const content = fs.readFileSync(filePath, 'utf-8');
    const matches = content.match(new RegExp(pattern, 'g'));
    return matches ? matches.length : 0;
  };

  expect(countInFile('webpack.js', '__webpack_require__\\.x\\(\\)')).toBe(0);
  expect(countInFile('other.js', '__webpack_require__\\.x\\(\\)')).toBe(0);
});

it('should have correct __webpack_require__.x distribution in Module Federation container', () => {
  const distPath = __dirname;
  const isESM = distPath.includes('/module') || distPath.includes('\\module');
  const ext = isESM ? '.mjs' : '.js';
  
  const countInFile = (filename, pattern) => {
    const filePath = path.join(distPath, filename.replace('.js', ext));
    const content = fs.readFileSync(filePath, 'utf-8');
    const matches = content.match(new RegExp(pattern, 'g'));
    return matches ? matches.length : 0;
  };

  expect(countInFile('container.js', '__webpack_require__\\.x\\(\\)')).toBe(1);
});

it('should have correct federation runtime distribution', () => {
  const distPath = __dirname;
  const isESM = distPath.includes('/module') || distPath.includes('\\module');
  const ext = isESM ? '.mjs' : '.js';
  
  const countInFile = (filename, pattern) => {
    const filePath = path.join(distPath, filename.replace('.js', ext));
    const content = fs.readFileSync(filePath, 'utf-8');
    const matches = content.match(new RegExp(pattern.replace('_MARK_', ''), 'g'));
    return matches ? matches.length : 0;
  };

  expect(countInFile('webpack.js', 'embed_feder_MARK_ation_runtime')).toBe(1);
  expect(countInFile('other.js', 'embed_federatio_MARK_n_runtime')).toBe(1);

  expect(countInFile('container.js', 'embed_federat_MARK_ion_runtime')).toBe(1);

  expect(countInFile('another.js', 'embed_fe_MARK_deration_runtime')).toBe(0);
  expect(countInFile('main.js', 'embed__MARK_federation_runtime')).toBe(0);
});
