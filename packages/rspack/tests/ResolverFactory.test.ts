import { promisify } from 'node:util';
import { rspack, type Compiler } from '@rspack/core';
import { beforeEach, describe, expect, it } from 'rstack/test';

let factory: Compiler['resolverFactory'];

beforeEach(() => {
  const compiler = rspack({});
  factory = compiler.resolverFactory;
  return () => promisify(compiler.close.bind(compiler))();
});

describe('ResolverFactory.get', () => {
  it('reuses resolvers for equivalent options', () => {
    expect(factory.get('normal')).toBe(factory.get('normal'));
    expect(factory.get('normal', { alias: { foo: './index.js' } })).toBe(
      factory.get('normal', { alias: { foo: './index.js' } }),
    );
  });

  it('creates distinct resolvers for different aliases', () => {
    expect(factory.get('normal', { alias: { foo: './index.js' } })).not.toBe(
      factory.get('normal', { alias: { bar: './index.js' } }),
    );
  });
});

describe('Resolver.withOptions', () => {
  it('reuses child resolvers for equivalent options', () => {
    const child = factory
      .get('normal')
      .withOptions({ alias: { foo: './index.js' } });
    expect(
      factory.get('normal').withOptions({ alias: { foo: './index.js' } }),
    ).toBe(child);
  });

  it('reuses child resolvers with the same merged options', () => {
    const parent = factory.get('normal', { alias: { foo: './index.js' } });
    expect(parent.withOptions({})).toBe(
      factory.get('normal').withOptions({ alias: { foo: './index.js' } }),
    );
  });
});
