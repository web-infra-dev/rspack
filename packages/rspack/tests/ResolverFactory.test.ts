import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
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

describe('Resolver.resolve', () => {
  it('preserves file and missing dependencies across error, ignored, and success results', async () => {
    const tempDir = fs.realpathSync(
      fs.mkdtempSync(path.join(os.tmpdir(), 'rspack-resolver-deps-')),
    );
    try {
      const existingFile = path.join(tempDir, 'existing.js');
      const ignoredFile = path.join(tempDir, 'ignored.js');
      const packageJson = path.join(tempDir, 'package.json');
      const missingTarget = path.join(tempDir, 'missing-target');

      fs.writeFileSync(existingFile, 'export default 1;\n');
      fs.writeFileSync(ignoredFile, 'export default 2;\n');
      fs.writeFileSync(
        packageJson,
        JSON.stringify({
          name: 'resolver-deps-test',
          browser: {
            './ignored.js': false,
          },
        }),
      );

      const resolver = factory.get('normal', { aliasFields: ['browser'] });
      const resolveWithContext = (request: string) => {
        const fileDependencies = new Set<string>();
        const missingDependencies = new Set<string>();
        return new Promise<{
          error: Error | null;
          result: string | false | undefined;
          req: Record<string, unknown> | undefined;
          fileDependencies: Set<string>;
          missingDependencies: Set<string>;
        }>((resolvePromise) => {
          resolver.resolve(
            {},
            tempDir,
            request,
            {
              fileDependencies: {
                add: (dep) => fileDependencies.add(dep),
              },
              missingDependencies: {
                add: (dep) => missingDependencies.add(dep),
              },
            },
            (error, result, req) => {
              resolvePromise({
                error,
                result,
                req: req as Record<string, unknown> | undefined,
                fileDependencies,
                missingDependencies,
              });
            },
          );
        });
      };

      const missingRes = await resolveWithContext('./missing-target');
      expect(missingRes.error).toBeInstanceOf(Error);
      expect(missingRes.missingDependencies.has(missingTarget)).toBe(true);

      const ignoredRes = await resolveWithContext('./ignored.js');
      expect(ignoredRes.error).toBeNull();
      expect(ignoredRes.result).toBe(false);
      expect(ignoredRes.req).toBeUndefined();
      expect(ignoredRes.fileDependencies.has(packageJson)).toBe(true);

      const successRes = await resolveWithContext('./existing.js');
      expect(successRes.error).toBeNull();
      expect(successRes.result).toBe(existingFile);
      expect(successRes.fileDependencies.has(existingFile)).toBe(true);
      expect(Array.isArray(successRes.req?.fileDependencies)).toBe(true);
      expect(Array.isArray(successRes.req?.missingDependencies)).toBe(true);
    } finally {
      fs.rmSync(tempDir, { recursive: true, force: true });
    }
  });
});
