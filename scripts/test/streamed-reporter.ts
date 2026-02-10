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
    const timestamp = new Date().toISOString();
    this.stream.write(`[${timestamp}] ${message}\n`);
  }

  onTestRunStart() {
    this.write('TEST_RUN_START');
  }

  onTestFileStart(file: TestFileInfo) {
    this.write(`FILE_START: ${file.testPath}`);
  }

  onTestFileReady(file: TestFileInfo) {
    this.write(`FILE_READY: ${file.testPath}`);
  }

  onTestSuiteStart(suite: TestSuiteInfo) {
    const suitePath = suite.parentNames
      ? `${suite.parentNames.join(' > ')} > ${suite.name}`
      : suite.name;
    this.write(`SUITE_START: ${suite.testPath} > ${suitePath}`);
  }

  onTestSuiteResult(result: TestResult) {
    const suitePath = result.parentNames
      ? `${result.parentNames.join(' > ')} > ${result.name}`
      : result.name;
    this.write(
      `SUITE_END: ${result.testPath} > ${suitePath} [${result.status}] (${result.duration}ms)`,
    );
  }

  onTestCaseStart(test: TestCaseInfo) {
    const testPath = test.parentNames
      ? `${test.parentNames.join(' > ')} > ${test.name}`
      : test.name;
    this.write(`TEST_START: ${test.testPath} > ${testPath}`);
  }

  onTestCaseResult(result: TestResult) {
    const testPath = result.parentNames
      ? `${result.parentNames.join(' > ')} > ${result.name}`
      : result.name;
    this.write(
      `TEST_END: ${result.testPath} > ${testPath} [${result.status}] (${result.duration}ms)`,
    );
    if (result.errors && result.errors.length > 0) {
      this.write(
        `TEST_ERROR: ${result.testPath} > ${testPath}: ${result.errors[0].message}`,
      );
    }
  }

  async onTestRunEnd() {
    this.write('TEST_RUN_END');
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
