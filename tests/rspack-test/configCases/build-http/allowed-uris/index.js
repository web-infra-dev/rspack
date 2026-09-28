import host from 'http://allowed.example/module.js';
import path from 'http://path.example/modules/module.js';
import regex from 'http://REGEX.EXAMPLE:80/module.js';
import redirected from 'http://allowed.example/redirect';

it('should normalize string rules and skip invalid rules', () => {
  expect(host).toBe('http://allowed.example/module.js');
  expect(path).toBe('http://path.example/modules/module.js');
});

it('should match regular expressions against normalized URLs', () => {
  expect(regex).toBe('http://regex.example/module.js');
});

it('should allow redirects that match a normalized rule', () => {
  expect(redirected).toBe('http://allowed.example/redirected.js');
});
