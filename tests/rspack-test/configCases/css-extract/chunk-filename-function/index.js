it('should pass chunk to css extract chunkFilename function', async () => {
  await expect(
    import(/* webpackChunkName: "async" */ './async'),
  ).resolves.toMatchObject({ value: 1 });

  const fs = require('fs');
  const css = fs.readFileSync(
    `${__STATS__.outputPath}/expected.async.css`,
    'utf-8',
  );
  const source = fs.readFileSync(`${__STATS__.outputPath}/main.js`, 'utf-8');

  expect(css).toContain('color: red');
  expect(source).toContain('expected.async.css');
});
