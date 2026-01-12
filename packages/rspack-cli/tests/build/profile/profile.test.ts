import fs from 'fs';
import { resolve } from 'path';
import { run } from '../../utils/test-utils';

const defaultTracePath = './rspack.pftrace';
const customTracePath = './custom/trace.json';

function findDefaultOutputDirname() {
  const files = fs.readdirSync(__dirname);
  const file = files.filter((file) => file.startsWith('.rspack-profile'));
  return file.length > 0 ? resolve(__dirname, file[0]) : null;
}

describe('profile', () => {
  afterEach(() => {
    const dirname = findDefaultOutputDirname();
    [dirname, resolve(__dirname, customTracePath)].forEach((p) => {
      if (p && fs.existsSync(p)) {
        fs.rmSync(p, { recursive: true });
      }
    });
  });

  it('should store all profile files when RSPACK_PROFILE=ALL enabled', async () => {
    const { exitCode } = await run(
      __dirname,
      [],
      {},
      { RSPACK_PROFILE: 'ALL' },
    );
    expect(exitCode).toBe(0);
    const dirname = findDefaultOutputDirname();
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
    const dirname = findDefaultOutputDirname();
    expect(fs.existsSync(resolve(dirname, defaultTracePath))).toBeTruthy();
  });

  it('should filter trace event when use RSPACK_PROFILE=rspack_resolver,rspack', async () => {
    const { exitCode } = await run(
      __dirname,
      [],
      {},
      {
        NO_COLOR: '1',
        RSPACK_PROFILE: 'rspack,respack_resolver',
        RSPACK_TRACE_OUTPUT: defaultTracePath,
        RSPACK_TRACE_LAYER: 'logger',
      },
    );
    expect(exitCode).toBe(0);
    const dirname = findDefaultOutputDirname();
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
    const dirname = findDefaultOutputDirname();
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
