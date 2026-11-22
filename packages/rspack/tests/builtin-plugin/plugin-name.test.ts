import { DefinePlugin, ProvidePlugin } from '@rspack/core';
import { expect, it } from 'rstack/test';

it('preserves the class names of built-in plugins', () => {
  expect(new DefinePlugin({}).constructor.name).toBe('DefinePlugin');
  expect(new ProvidePlugin({}).constructor.name).toBe('ProvidePlugin');
});
