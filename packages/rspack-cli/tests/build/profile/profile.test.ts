import fs from 'fs';
import { resolve } from 'path';
import { run } from '../../utils/test-utils';

const defaultLoggerTracePath = 'rspack.log';
const customTracePath = './custom/trace.json';

function findDefaultOutputDirname() {
  const files = fs.readdirSync(__dirname);
  const file = files.filter((file) => file.startsWith('.rspack-profile'));
  return file.length > 0 ? resolve(__dirname, file[0]) : null;
}

function getDefaultOutputDirname() {
  const dirname = findDefaultOutputDirname();
  if (!dirname) {
    throw new Error('Expected a .rspack-profile-* directory to be created');
  }
  return dirname;
}

function cleanupProfileOutput() {
  const profileDirs = fs
    .readdirSync(__dirname)
    .filter((file) => file.startsWith('.rspack-profile'))
    .map((file) => resolve(__dirname, file));
  [...profileDirs, resolve(__dirname, customTracePath)].forEach((p) => {
    if (p && fs.existsSync(p)) {
      fs.rmSync(p, { recursive: true });
    }
  });
}

describe('profile', () => {
  beforeEach(cleanupProfileOutput);
  afterEach(cleanupProfileOutput);

  it('should store all profile files when RSPACK_PROFILE=ALL enabled', async () => {
    const { exitCode, stdout } = await run(
      __dirname,
      [],
      {},
      { RSPACK_PROFILE: 'ALL' },
    );
    expect(exitCode).toBe(0);
    expect(stdout.includes('"target":"rspack_binding_api"')).toBe(false);

    const dirname = getDefaultOutputDirname();
    const tracePath = resolve(dirname, defaultLoggerTracePath);
    expect(fs.existsSync(tracePath)).toBeTruthy();
    const content = fs.readFileSync(tracePath, 'utf-8');
    expect(content.includes('"target":"rspack_binding_api"')).toBe(true);
  });

  it('should store rust trace file when RSPACK_PROFILE=OVERVIEW enabled', async () => {
    const { exitCode, stdout } = await run(
      __dirname,
      [],
      {},
      { RSPACK_PROFILE: 'OVERVIEW' },
    );
    expect(exitCode).toBe(0);
    expect(stdout.includes('"target":"rspack_binding_api"')).toBe(false);

    const dirname = getDefaultOutputDirname();
    const tracePath = resolve(dirname, defaultLoggerTracePath);
    expect(fs.existsSync(tracePath)).toBeTruthy();
    const content = fs.readFileSync(tracePath, 'utf-8');
    expect(content.includes('"target":"rspack_binding_api"')).toBe(true);
  });

  it('should filter trace event when use RSPACK_PROFILE=rspack_resolver,rspack', async () => {
    const { exitCode } = await run(
      __dirname,
      [],
      {},
      {
        NO_COLOR: '1',
        RSPACK_PROFILE: 'rspack,respack_resolver',
        RSPACK_TRACE_OUTPUT: defaultLoggerTracePath,
        RSPACK_TRACE_LAYER: 'logger',
      },
    );
    expect(exitCode).toBe(0);
    const dirname = getDefaultOutputDirname();
    const tracePath = resolve(dirname, defaultLoggerTracePath);
    expect(fs.existsSync(tracePath)).toBeTruthy();
    const content = fs.readFileSync(tracePath, 'utf-8');
    const out: any[] = content
      .trim()
      .split('\n')
      .map((line) => {
        return JSON.parse(line);
      });

    expect(
      out
        .filter((line) => line.target)
        .every(
          (line) =>
            line.target.startsWith('rspack') ||
            line.target.startsWith('rspack_resolver') ||
            line.target.startsWith('javascript'),
        ),
    ).toBe(true);
  });

  it('should be able to customize output path', async () => {
    const { exitCode } = await run(
      __dirname,
      [],
      {},
      {
        RSPACK_PROFILE: 'OVERVIEW',
        RSPACK_TRACE_OUTPUT: customTracePath,
      },
    );
    expect(exitCode).toBe(0);
    const dirname = getDefaultOutputDirname();
    expect(fs.existsSync(resolve(dirname, customTracePath))).toBeTruthy();
  });

  it('should be able to use logger trace layer and default output should be file', async () => {
    const { exitCode, stdout } = await run(
      __dirname,
      [],
      {},
      {
        RSPACK_PROFILE: `rspack_core::compiler`,
        RSPACK_TRACE_LAYER: 'logger',
      },
    );
    expect(exitCode).toBe(0);
    expect(stdout.includes('rspack_core::compiler')).toBe(false);

    const dirname = getDefaultOutputDirname();
    const tracePath = resolve(dirname, defaultLoggerTracePath);
    expect(fs.existsSync(tracePath)).toBeTruthy();
    const content = fs.readFileSync(tracePath, 'utf-8');
    expect(content.includes('rspack_core::compiler')).toBe(true);
  });

  it('should reject terminal output for perfetto trace layer', async () => {
    const { exitCode, stderr } = await run(
      __dirname,
      [],
      {},
      {
        RSPACK_PROFILE: 'OVERVIEW',
        RSPACK_TRACE_LAYER: 'perfetto',
        RSPACK_TRACE_OUTPUT: 'stdout',
      },
    );
    expect(exitCode).toBe(1);
    expect(stderr).toContain(
      'RSPACK_TRACE_OUTPUT=stdout|stderr is only supported for the logger trace layer',
    );
  });
});
