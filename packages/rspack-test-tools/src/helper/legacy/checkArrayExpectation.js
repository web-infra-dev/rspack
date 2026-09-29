// @ts-nocheck
import fs from 'fs-extra';
import path from 'node:path';

export function findExpectationFile(testDirectory, filename) {
  return ['mjs', 'cjs', 'js']
    .map((extension) => path.join(testDirectory, `${filename}.${extension}`))
    .find((file) => fs.existsSync(file));
}

function readExpectationFile(file) {
  const expected = require(file);
  return file.endsWith('.mjs') ? expected.default : expected;
}

const check = (expected, actual) => {
  if (expected instanceof RegExp) {
    expected = { message: expected };
  }
  if (Array.isArray(expected)) {
    return expected.every((e) => check(e, actual));
  }
  return Object.keys(expected).every((key) => {
    let value = actual[key];
    if (typeof value === 'object') {
      value = JSON.stringify(value);
    }
    return expected[key].test(value);
  });
};

const explain = (object) => {
  if (object instanceof RegExp) {
    object = { message: object };
  }
  return Object.keys(object)
    .map((key) => {
      let value = object[key];
      if (typeof value === 'object' && !(value instanceof RegExp)) {
        value = JSON.stringify(value);
      }
      let msg = `${key} = ${value}`;
      if (key !== 'stack' && key !== 'details' && msg.length > 600)
        msg = msg.slice(0, 597) + '...';
      return msg;
    })
    .join('; ');
};

const diffItems = (actual, expected, kind) => {
  const tooMuch = actual.slice();
  const missing = expected.slice();
  for (let i = 0; i < missing.length; i++) {
    const current = missing[i];
    for (let j = 0; j < tooMuch.length; j++) {
      if (check(current, tooMuch[j])) {
        tooMuch.splice(j, 1);
        missing.splice(i, 1);
        i--;
        break;
      }
    }
  }
  const diff = [];
  if (missing.length > 0) {
    diff.push(`The following expected ${kind}s are missing:
${missing.map((item) => `${explain(item)}`).join('\n\n')}`);
  }
  if (tooMuch.length > 0) {
    diff.push(`The following ${kind}s are unexpected:
${tooMuch.map((item) => `${explain(item)}`).join('\n\n')}`);
  }
  return diff.join('\n\n');
};

// Keep the promise-returning contract of this publicly exported legacy helper.
// eslint-disable-next-line @typescript-eslint/require-await
export async function checkArrayExpectation(
  testDirectory,
  object,
  kind,
  filename,
  upperCaseKind,
  options,
  done,
) {
  done =
    typeof done === 'function'
      ? done
      : (error) => {
          throw error;
        };
  let array = object[`${kind}s`];
  if (Array.isArray(array) && kind === 'warning') {
    array = array.filter((item) => !/from Terser/.test(item));
  }
  const expectedFilename = findExpectationFile(testDirectory, filename);
  if (expectedFilename) {
    // CHANGE: added file for sorting messages in multi-thread environment
    const sorterFilename = findExpectationFile(
      testDirectory,
      `${filename}-sort`,
    );
    if (sorterFilename) {
      const sorter = readExpectationFile(sorterFilename);
      array = sorter(array);
    }
    let expected = readExpectationFile(expectedFilename);
    if (typeof expected === 'function') {
      expected = expected(options);
    }
    const diff = diffItems(array, expected, kind);

    if (expected.length < array.length) {
      done(
        new Error(
          `More ${kind}s (${array.length} instead of ${expected.length}) while compiling than expected:\n\n${diff}\n\nCheck expected ${kind}s: ${expectedFilename}`,
        ),
      );

      return true;
    }
    if (expected.length > array.length) {
      done(
        new Error(
          `Less ${kind}s (${array.length} instead of ${expected.length}) while compiling than expected:\n\n${diff}\n\nCheck expected ${kind}s: ${expectedFilename}`,
        ),
      );
      return true;
    }

    const usedExpected = new Array(expected.length).fill(false);

    for (let i = 0; i < array.length; i++) {
      let found = false;
      for (let j = 0; j < expected.length; j++) {
        if (usedExpected[j]) continue;

        if (Array.isArray(expected[j])) {
          for (let k = 0; k < expected[j].length; k++) {
            if (check(expected[j][k], array[i])) {
              usedExpected[j] = true;
              found = true;
              break;
            }
          }
        } else {
          if (check(expected[j], array[i])) {
            usedExpected[j] = true;
            found = true;
            break;
          }
        }
        if (found) break;
      }
      if (!found) {
        done(
          new Error(
            `${upperCaseKind} ${i}: ${explain(array[i])} doesn't match any expected value`,
          ),
        );
        return true;
      }
    }

    const unused = [];
    for (let j = 0; j < expected.length; j++) {
      if (!usedExpected[j]) {
        unused.push(
          Array.isArray(expected[j])
            ? expected[j].map(explain).join(' | ')
            : explain(expected[j]),
        );
      }
    }
    if (unused.length > 0) {
      done(
        new Error(
          `The following expected ${kind}s were not matched:\n${unused
            .map((u) => `  ${u}`)
            .join('\n')}`,
        ),
      );
      return true;
    }
  } else if (array.length > 0) {
    done(
      new Error(
        `${upperCaseKind}s while compiling:\n\n${array
          .map(explain)
          .join('\n\n')}`,
      ),
    );
    return true;
  }
}
