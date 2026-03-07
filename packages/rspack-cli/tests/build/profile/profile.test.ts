import fs from 'fs';
import { resolve } from 'path';
import { run } from '../../utils/test-utils';

const defaultTracePath = './rspack.pftrace';
const customTracePath = './custom/trace.json';
const hotpathJsonPath = './custom/hotpath.json';

function findDefaultOutputDirname() {
  const files = fs.readdirSync(__dirname);
  const file = files.filter((file) => file.startsWith('.rspack-profile'));
  return file.length > 0 ? resolve(__dirname, file[0]) : null;
}

describe('profile', () => {
  afterEach(() => {
    const dirname = findDefaultOutputDirname();
    [
      dirname,
      resolve(__dirname, customTracePath),
      resolve(__dirname, hotpathJsonPath),
    ].forEach((p) => {
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

  it('should be able to use hotpath trace layer and print an aggregated table to stdout', async () => {
    const { exitCode, stdout } = await run(
      __dirname,
      [],
      {},
      {
        NO_COLOR: '1',
        RSPACK_PROFILE: 'rspack_core::compiler',
        RSPACK_TRACE_LAYER: 'hotpath',
      },
    );

    expect(exitCode).toBe(0);
    expect(stdout).toContain('Calls');
    expect(stdout).toContain('Avg');
    expect(stdout).toContain('P95');
    expect(stdout).toContain('Total');
    expect(stdout).toContain('% Total');
    expect(stdout).toContain('Overall elapsed');
  });

  it('should be able to use hotpath trace layer and emit json output for diffing', async () => {
    const { exitCode } = await run(
      __dirname,
      [],
      {},
      {
        NO_COLOR: '1',
        RSPACK_PROFILE: 'rspack_core::compiler',
        RSPACK_TRACE_LAYER: 'hotpath',
        RSPACK_TRACE_OUTPUT: hotpathJsonPath,
      },
    );

    expect(exitCode).toBe(0);
    const dirname = findDefaultOutputDirname();
    const tracePath = resolve(dirname, hotpathJsonPath);
    expect(fs.existsSync(tracePath)).toBeTruthy();

    const report = JSON.parse(fs.readFileSync(tracePath, 'utf-8'));
    expect(report.rspack_trace_layer).toBe('hotpath');
    expect(report.output_format).toBe('json');
    expect(report.total_elapsed_ns).toBeGreaterThan(0);
    expect(report.percentiles).toEqual([95]);
    expect(Array.isArray(report.data)).toBe(true);
    expect(report.data.length).toBeGreaterThan(0);
    expect(report.data[0]).toEqual(
      expect.objectContaining({
        name: expect.any(String),
        calls: expect.any(Number),
        avg_raw: expect.any(Number),
        total_raw: expect.any(Number),
        percent_total_raw: expect.any(Number),
      }),
    );
    expect(report.data[0].percentiles_raw).toEqual(
      expect.objectContaining({
        p95: expect.any(Number),
      }),
    );
  });
});
