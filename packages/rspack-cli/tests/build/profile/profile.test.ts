import fs from 'fs';
import { resolve } from 'path';
import { run } from '../../utils/test-utils';

const defaultTracePath = './rspack.pftrace';
const customTracePath = './custom/trace.json';

function getProfileOutputDirs() {
  return fs
    .readdirSync(__dirname)
    .filter((file) => file.startsWith('.rspack-profile'))
    .map((file) => resolve(__dirname, file))
    .sort((a, b) => fs.statSync(b).mtimeMs - fs.statSync(a).mtimeMs);
}

function findOutputDirname(tracePath = defaultTracePath) {
  return (
    getProfileOutputDirs().find((dir) =>
      fs.existsSync(resolve(dir, tracePath)),
    ) || null
  );
}

function cleanupProfileOutputs() {
  [
    ...getProfileOutputDirs(),
    resolve(__dirname, defaultTracePath),
    resolve(__dirname, customTracePath),
  ].forEach((p) => {
    if (fs.existsSync(p)) {
      fs.rmSync(p, { recursive: true, force: true });
    }
  });
}

describe('profile', () => {
  beforeEach(() => {
    cleanupProfileOutputs();
  });

  afterEach(() => {
    cleanupProfileOutputs();
  });

  it('should store all profile files when RSPACK_PROFILE=ALL enabled', async () => {
    const { exitCode } = await run(
      __dirname,
      [],
      {},
      { RSPACK_PROFILE: 'ALL' },
    );
    expect(exitCode).toBe(0);
    const dirname = findOutputDirname();
    expect(dirname).toBeTruthy();
    expect(fs.existsSync(resolve(dirname, defaultTracePath))).toBeTruthy();
  });

  it('should store rust trace file when RSPACK_PROFILE=OVERVIEW enabled', async () => {
    const { exitCode } = await run(
      __dirname,
      [],
      {},
      { RSPACK_PROFILE: 'OVERVIEW' },
    );
    expect(exitCode).toBe(0);
    const dirname = findOutputDirname();
    expect(dirname).toBeTruthy();
    expect(fs.existsSync(resolve(dirname, defaultTracePath))).toBeTruthy();
  });

  it('should filter trace event when use RSPACK_PROFILE=rspack_resolver,rspack', async () => {
    const { exitCode } = await run(
      __dirname,
      [],
      {},
      {
        NO_COLOR: '1',
        RSPACK_PROFILE: 'rspack,rspack_resolver',
        RSPACK_TRACE_OUTPUT: defaultTracePath,
        RSPACK_TRACE_LAYER: 'logger',
      },
    );
    expect(exitCode).toBe(0);
    const dirname = findOutputDirname();
    expect(dirname).toBeTruthy();
    const tracePath = resolve(dirname, defaultTracePath);
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
    const dirname = findOutputDirname(customTracePath);
    expect(dirname).toBeTruthy();
    expect(fs.existsSync(resolve(dirname, customTracePath))).toBeTruthy();
  });

  it('should be able to use logger trace layer and default output should be stdout', async () => {
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
    expect(stdout.includes('rspack_core::compiler')).toBe(true);
  });
});
