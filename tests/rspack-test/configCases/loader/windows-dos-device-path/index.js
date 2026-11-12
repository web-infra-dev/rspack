const value = require('./resource.js?resource-query#fragment');

it('should build modules through a Windows DOS device path', () => {
  expect(value).toBe('resource');
});
