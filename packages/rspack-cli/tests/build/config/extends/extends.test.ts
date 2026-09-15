import { readFile } from 'fs/promises';
import { resolve } from 'path';
import { run } from '../../../utils/test-utils';

describe('rspack extends feature', () => {
  describe('basic extends', () => {
    const cwd = resolve(__dirname, './base');

    it('should extend from base config', async () => {
      const { exitCode, stderr, stdout } = await run(cwd, []);

      expect(stderr).toBeFalsy();
      expect(stdout).toBeTruthy();
      expect(exitCode).toBe(0);

      // Check if the output file has the correct name (from base config)
      const outputContent = await readFile(
        resolve(cwd, './dist/base.bundle.js'),
        {
          encoding: 'utf-8',
        },
      );

      expect(outputContent).toMatch(/Base extends test/);
    });
  });

  describe('nested extends', () => {
    const cwd = resolve(__dirname, './nested');

    it('should extend from nested configs', async () => {
      const { exitCode, stderr, stdout } = await run(cwd, []);

      expect(stderr).toBeFalsy();
      expect(stdout).toBeTruthy();
      expect(exitCode).toBe(0);

      // Check if the output file has the correct name (from base config)
      const outputContent = await readFile(
        resolve(cwd, './dist/base.bundle.js'),
        {
          encoding: 'utf-8',
        },
      );

      expect(outputContent).toMatch(/Nested extends test/);
    });
  });

  describe('multiple extends', () => {
    const cwd = resolve(__dirname, './multiple');

    it('should extend from multiple configs', async () => {
      const { exitCode, stderr, stdout } = await run(cwd, []);

      expect(stderr).toBeFalsy();
      expect(stdout).toBeTruthy();
      expect(exitCode).toBe(0);

      // Check if the output file has the correct name (from base config)
      const outputContent = await readFile(
        resolve(cwd, './dist/base.bundle.js'),
        {
          encoding: 'utf-8',
        },
      );

      expect(outputContent).toMatch(/Multiple extends test/);

      // Check if the devtool from dev config is applied
      expect(outputContent).toMatch(/eval-source-map/);
    });
  });

  describe('function config with extends', () => {
    const cwd = resolve(__dirname, './function');

    it('should handle extends in function configs', async () => {
      const { exitCode, stderr, stdout } = await run(cwd, []);

      expect(stderr).toBeFalsy();
      expect(stdout).toBeTruthy();
      expect(exitCode).toBe(0);

      // Check if the output file has the correct name (from base config)
      const outputContent = await readFile(
        resolve(cwd, './dist/base.bundle.js'),
        {
          encoding: 'utf-8',
        },
      );

      expect(outputContent).toMatch(/Function extends test/);
    });
  });

  describe('node module extends', () => {
    const cwd = resolve(__dirname, './node-module');

    it('should extend from a node module', async () => {
      const { exitCode, stderr, stdout } = await run(cwd, []);

      expect(stderr).toBeFalsy();
      expect(stdout).toBeTruthy();
      expect(exitCode).toBe(0);

      // Check if the output file has the correct name (from node module config)
      const outputContent = await readFile(
        resolve(cwd, './dist/node-module.bundle.js'),
        {
          encoding: 'utf-8',
        },
      );

      expect(outputContent).toMatch(/Node module extends test/);
    });
  });

  describe('file protocol extends', () => {
    const cwd = resolve(__dirname, './file-protocol');

    it('should extend from a config using file protocol (import.meta.resolve)', async () => {
      const { exitCode, stderr, stdout } = await run(cwd, []);

      expect(stderr).toBeFalsy();
      expect(stdout).toBeTruthy();
      expect(exitCode).toBe(0);

      const outputContent = await readFile(
        resolve(cwd, './dist/base.bundle.js'),
        {
          encoding: 'utf-8',
        },
      );

      expect(outputContent).toMatch(/File protocol extends test/);
    });
  });

  describe('require.resolve extends', () => {
    const cwd = resolve(__dirname, './require-resolve');

    it('should extend from a config using require.resolve', async () => {
      const { exitCode, stderr, stdout } = await run(cwd, []);

      expect(stderr).toBeFalsy();
      expect(stdout).toBeTruthy();
      expect(exitCode).toBe(0);

      const outputContent = await readFile(
        resolve(cwd, './dist/base.bundle.js'),
        {
          encoding: 'utf-8',
        },
      );

      expect(outputContent).toMatch(/Require resolve extends test/);
    });
  });

  describe('json config extends', () => {
    const cwd = resolve(__dirname, './json');

    it('should extend from a JSON config file', async () => {
      const { exitCode, stderr, stdout } = await run(cwd, []);

      expect(stderr).toBeFalsy();
      expect(stdout).toBeTruthy();
      expect(exitCode).toBe(0);

      const outputContent = await readFile(
        resolve(cwd, './dist/base.bundle.js'),
        {
          encoding: 'utf-8',
        },
      );

      expect(outputContent).toMatch(/JSON extends test/);
    });
  });

  describe('multiple configs with extends', () => {
    const cwd = resolve(__dirname, './multiple-configs');

    it('should extend base config for each config in a multi-config array', async () => {
      const { exitCode, stderr, stdout } = await run(cwd, []);

      expect(stderr).toBeFalsy();
      expect(stdout).toBeTruthy();
      expect(exitCode).toBe(0);

      const output1 = await readFile(resolve(cwd, './dist/bundle1.js'), {
        encoding: 'utf-8',
      });
      const output2 = await readFile(resolve(cwd, './dist/bundle2.js'), {
        encoding: 'utf-8',
      });

      expect(output1).toMatch(/i am index1/);
      expect(output2).toMatch(/i am index2/);
    });
  });

  describe('multiple configs with partial extends', () => {
    const cwd = resolve(__dirname, './multiple-configs2');

    it('should extend base config only for configs that have extends property', async () => {
      const { exitCode, stderr, stdout } = await run(cwd, []);

      expect(stderr).toBeFalsy();
      expect(stdout).toBeTruthy();
      expect(exitCode).toBe(0);

      const output1 = await readFile(resolve(cwd, './dist/bundle1.js'), {
        encoding: 'utf-8',
      });
      const output2 = await readFile(resolve(cwd, './dist/bundle2.js'), {
        encoding: 'utf-8',
      });

      expect(output1).toMatch(/i am index1/);
      expect(output2).toMatch(/i am index2/);
    });
  });

  describe('recursive extends', () => {
    const cwd = resolve(__dirname, './recursive-extends');

    it('should throw an error on recursive extends', async () => {
      const { exitCode, stderr } = await run(cwd, []);

      expect(exitCode).not.toBe(0);
      expect(stderr).toBeTruthy();
    });

    it('should throw an error on recursive extends #2', async () => {
      const { exitCode, stderr } = await run(cwd, [
        '--config',
        'other.config.js',
      ]);

      expect(exitCode).not.toBe(0);
      expect(stderr).toBeTruthy();
    });
  });

  describe('backward compatibility', () => {
    // Use an existing test directory that doesn't use the extends feature
    const cwd = resolve(__dirname, '../../basic');

    it('should not break existing functionality', async () => {
      const { exitCode, stderr, stdout } = await run(cwd, [
        '--config',
        './entry.config.js',
      ]);

      expect(stderr).toBeFalsy();
      expect(stdout).toBeTruthy();
      expect(exitCode).toBe(0);

      // Check if the output file is generated correctly
      const outputExists = await readFile(resolve(cwd, './dist/main.js'), {
        encoding: 'utf-8',
      })
        .then(() => true)
        .catch(() => false);

      expect(outputExists).toBe(true);
    });
  });

  describe('error handling', () => {
    const cwd = resolve(__dirname);

    it('should throw an error when extended config file is not found', async () => {
      const { exitCode, stderr } = await run(cwd, [
        '-c',
        './error/not-found.config.js',
      ]);

      expect(exitCode).not.toBe(0);
      expect(stderr).toMatch(/not found/);
    });
  });
});
