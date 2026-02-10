import * as fs from 'node:fs';
import * as path from 'node:path';
import type {
  Reporter,
  TestCaseInfo,
  TestFileInfo,
  TestResult,
  TestSuiteInfo,
} from '@rstest/core';

export class StreamedTextReporter implements Reporter {
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
