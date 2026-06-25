// @ts-nocheck

/* istanbul ignore file */

import fs from 'node:fs';
import path from 'node:path';
import chalk from 'chalk';
import filenamify from 'filenamify';
import { diff } from 'jest-diff';
import { serializers } from '../serializers';
import {
  getSnapshotSerializers,
  serializeSnapshot,
} from './snapshot-serializers';

/**
 * Check if 2 strings or buffer are equal
 */
const isEqual = (a: string | Buffer, b: string | Buffer): boolean => {
  // @ts-expect-error: TypeScript gives error if we pass string to buffer.equals
  return Buffer.isBuffer(a) ? a.equals(b) : a === b;
};

function readSnapshot(filename: string, content: string | Buffer) {
  const output = fs.readFileSync(
    filename,
    Buffer.isBuffer(content) ? null : 'utf8',
  );
  return Buffer.isBuffer(output) ? output : output.replace(/\r\n/g, '\n');
}

function toPosixPath(filename: string) {
  return filename.split(path.sep).join('/');
}

function getRuntimeModeSnapshotFilename(filename: string): string | undefined {
  if (
    !(globalThis as { __RSPACK_TEST_RUNTIME_MODE_RSPACK?: boolean })
      .__RSPACK_TEST_RUNTIME_MODE_RSPACK
  ) {
    return;
  }

  const normalized = toPosixPath(filename);
  if (normalized.includes('/runtimeModeSnapshot/')) {
    return;
  }

  for (const marker of ['/__snapshot__/', '/__snapshots__/']) {
    const markerIndex = normalized.indexOf(marker);
    if (markerIndex >= 0) {
      return path.normalize(
        `${normalized.slice(0, markerIndex + marker.length)}runtimeModeSnapshot/${normalized.slice(markerIndex + marker.length)}`,
      );
    }
  }
}

function cleanupRuntimeModeSnapshot(filename: string) {
  fs.unlinkSync(filename);

  let current = path.dirname(filename);
  while (path.basename(current) !== 'runtimeModeSnapshot') {
    try {
      fs.rmdirSync(current);
    } catch {
      break;
    }
    current = path.dirname(current);
  }
}

/**
 * Match given content against content of the specified file.
 *
 * @param content Output content to match
 * @param filepath Path to the file to match against
 * @param options Additional options for matching
 */
export function toMatchFileSnapshotSync(
  this: {
    testPath: string;
    currentTestName: string;
    assertionCalls: number;
    isNot: boolean;
    snapshotState: {
      added: number;
      updated: number;
      unmatched: number;
      _updateSnapshot: 'none' | 'new' | 'all';
    };
  },
  rawContent: string | Buffer,
  filepath: string,
  options: FileMatcherOptions = {},
) {
  const content = Buffer.isBuffer(rawContent)
    ? rawContent
    : serializeSnapshot(rawContent, /* ident */ 2, {
        plugins: [
          ...getSnapshotSerializers(),
          // Rspack serializers
          ...serializers,
        ],
      });

  const { isNot, snapshotState } = this;

  const filename =
    filepath === undefined
      ? // If file name is not specified, generate one from the test title
        path.join(
          path.dirname(this.testPath),
          '__file_snapshots__',
          `${filenamify(this.currentTestName, {
            replacement: '-',
          }).replace(/\s/g, '-')}-${this.assertionCalls}`,
        )
      : filepath;
  const runtimeModeSnapshotFilename = getRuntimeModeSnapshotFilename(filename);
  const matchedFilename =
    runtimeModeSnapshotFilename && fs.existsSync(runtimeModeSnapshotFilename)
      ? runtimeModeSnapshotFilename
      : filename;

  if (
    snapshotState._updateSnapshot === 'none' &&
    !fs.existsSync(matchedFilename)
  ) {
    // We're probably running in CI environment

    snapshotState.unmatched++;

    return {
      pass: isNot,
      message: () =>
        `New output file ${chalk.blue(
          path.basename(matchedFilename),
        )} was ${chalk.bold.red('not written')}.\n\nThe update flag must be explicitly passed to write a new snapshot.\n\nThis is likely because this test is run in a ${chalk.blue(
          'continuous integration (CI) environment',
        )} in which snapshots are not written by default.\n\n`,
    };
  }

  if (fs.existsSync(matchedFilename)) {
    const output = readSnapshot(matchedFilename, content);

    if (isNot) {
      // The matcher is being used with `.not`

      if (!isEqual(content, output)) {
        // The value of `pass` is reversed when used with `.not`
        return { pass: false, message: () => '' };
      }
      snapshotState.unmatched++;

      return {
        pass: true,
        message: () =>
          `Expected received content ${chalk.red(
            'to not match',
          )} the file ${chalk.blue(path.basename(matchedFilename))}.`,
      };
    }
    if (isEqual(content, output)) {
      return { pass: true, message: () => '' };
    }
    if (snapshotState._updateSnapshot === 'all') {
      if (
        runtimeModeSnapshotFilename &&
        matchedFilename === runtimeModeSnapshotFilename &&
        fs.existsSync(filename) &&
        isEqual(content, readSnapshot(filename, content))
      ) {
        cleanupRuntimeModeSnapshot(runtimeModeSnapshotFilename);

        snapshotState.updated++;

        return { pass: true, message: () => '' };
      }

      const updatedFilename =
        runtimeModeSnapshotFilename && fs.existsSync(filename)
          ? runtimeModeSnapshotFilename
          : matchedFilename;
      fs.mkdirSync(path.dirname(updatedFilename), { recursive: true });
      fs.writeFileSync(updatedFilename, content);

      snapshotState.updated++;

      return { pass: true, message: () => '' };
    }
    snapshotState.unmatched++;

    const difference =
      Buffer.isBuffer(content) || Buffer.isBuffer(output)
        ? ''
        : `\n\n${diff(
            output,
            content,
            Object.assign(
              {
                expand: false,
                contextLines: 5,
                aAnnotation: 'Snapshot',
              },
              options.diff || {},
            ),
          )}`;

    return {
      pass: false,
      message: () =>
        `Received content ${chalk.red(
          "doesn't match",
        )} the file ${chalk.blue(path.basename(matchedFilename))}.${difference}`,
    };
  }
  if (
    !isNot &&
    (snapshotState._updateSnapshot === 'new' ||
      snapshotState._updateSnapshot === 'all')
  ) {
    if (
      runtimeModeSnapshotFilename &&
      fs.existsSync(filename) &&
      isEqual(content, readSnapshot(filename, content))
    ) {
      return { pass: true, message: () => '' };
    }

    const newFilename = runtimeModeSnapshotFilename || filename;
    fs.mkdirSync(path.dirname(newFilename), { recursive: true });
    fs.writeFileSync(newFilename, content);

    snapshotState.added++;

    return { pass: true, message: () => '' };
  }
  snapshotState.unmatched++;

  return {
    pass: true,
    message: () =>
      `The output file ${chalk.blue(
        path.basename(matchedFilename),
      )} ${chalk.bold.red("doesn't exist")}.`,
  };
}
