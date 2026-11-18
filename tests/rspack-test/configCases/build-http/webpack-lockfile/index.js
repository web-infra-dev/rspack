import asset from 'https://raw.githubusercontent.com/webpack/webpack/refs/heads/main/test/configCases/asset-modules/_images/file.svg';

it('bundles the asset from a webpack HTTP lockfile', () => {
  expect(asset).toMatch(/\.svg$/);
});
