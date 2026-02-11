import * as fs from 'node:fs';
import * as path from 'node:path';
import type {
  Reporter,
  TestCaseInfo,
  TestFileInfo,
  TestResult,
  TestSuiteInfo,
} from '@rstest/core';

/**
 * Stream Rstest events to a file in real-time for inspecting how tests are executed.
 *
 * Event format: `<context> | <event_type> | <timestamp> [| <additional_fields>]`
 *
 * @example
 * __GLOBAL__ | TEST_RUN_START | 2025-01-15T10:30:00.123Z
 * tests/unit/example.test.ts | FILE_START | 2025-01-15T10:30:00.456Z
 * tests/unit/example.test.ts | FILE_READY | 2025-01-15T10:30:00.789Z
 * tests/unit/example.test.ts > Suite Name | SUITE_START | 2025-01-15T10:30:01.000Z
 * tests/unit/example.test.ts > Suite Name > nested describe | SUITE_START | 2025-01-15T10:30:01.100Z
 * tests/unit/example.test.ts > Suite Name > nested describe > should work | TEST_START | 2025-01-15T10:30:01.200Z
 * tests/unit/example.test.ts > Suite Name > nested describe > should work | TEST_END | 2025-01-15T10:30:01.350Z | passed | 150ms
 * tests/unit/example.test.ts > Suite Name > nested describe | SUITE_END | 2025-01-15T10:30:01.400Z | passed | 300ms
 * tests/unit/example.test.ts > Suite Name | SUITE_END | 2025-01-15T10:30:01.500Z | passed | 500ms
 * __GLOBAL__ | TEST_RUN_END | 2025-01-15T10:30:02.000Z
 * */
export class StreamedEventReporter implements Reporter {
  private stream: fs.WriteStream;
  private outputPath: string;

  constructor(outputPath?: string) {
    this.outputPath =
      outputPath || path.join(process.cwd(), 'rstest-streamed-report.txt');
    const dir = path.dirname(this.outputPath);
    if (!fs.existsSync(dir)) {
      fs.mkdirSync(dir, { recursive: true });
    }
    this.stream = fs.createWriteStream(this.outputPath, { flags: 'w' });
  }

  private write(message: string): void {
    this.stream.write(`${message}\n`);
  }

  private formatTimestamp(): string {
    return new Date().toISOString();
  }

  onTestRunStart() {
    this.write(`__GLOBAL__ | TEST_RUN_START | ${this.formatTimestamp()}`);
  }

  onTestFileStart(file: TestFileInfo) {
    this.write(`${file.testPath} | FILE_START | ${this.formatTimestamp()}`);
  }

  onTestFileReady(file: TestFileInfo) {
    this.write(`${file.testPath} | FILE_READY | ${this.formatTimestamp()}`);
  }

  onTestSuiteStart(suite: TestSuiteInfo) {
    const suitePath = suite.parentNames
      ? `${suite.parentNames.join(' > ')} > ${suite.name}`
      : suite.name;
    this.write(
      `${suite.testPath} > ${suitePath} | SUITE_START | ${this.formatTimestamp()}`,
    );
  }

  onTestSuiteResult(result: TestResult) {
    const suitePath = result.parentNames
      ? `${result.parentNames.join(' > ')} > ${result.name}`
      : result.name;
    this.write(
      `${result.testPath} > ${suitePath} | SUITE_END | ${this.formatTimestamp()} | ${result.status} | ${result.duration}ms`,
    );
  }

  onTestCaseStart(test: TestCaseInfo) {
    const testPath = test.parentNames
      ? `${test.parentNames.join(' > ')} > ${test.name}`
      : test.name;
    this.write(
      `${test.testPath} > ${testPath} | TEST_START | ${this.formatTimestamp()}`,
    );
  }

  onTestCaseResult(result: TestResult) {
    const testPath = result.parentNames
      ? `${result.parentNames.join(' > ')} > ${result.name}`
      : result.name;
    this.write(
      `${result.testPath} > ${testPath} | TEST_END | ${this.formatTimestamp()} | ${result.status} | ${result.duration}ms`,
    );
    if (result.errors && result.errors.length > 0) {
      this.write(
        `${result.testPath} > ${testPath} | TEST_ERROR | ${this.formatTimestamp()} | ${result.errors[0].message}`,
      );
    }
  }

  async onTestRunEnd() {
    this.write(`__GLOBAL__ | TEST_RUN_END | ${this.formatTimestamp()}`);
    return new Promise<void>((resolve) => {
      this.stream.end(() => {
        resolve();
      });
    });
  }

  onExit() {
    this.stream.end();
  }
}
