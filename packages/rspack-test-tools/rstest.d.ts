/// <reference types="rstack/test/globals" />

import type { DiffOptions } from 'jest-diff';

declare interface FileMatcherOptions {
  diff?: DiffOptions;
}

declare module 'rstack/test' {
  interface Assertion {
    toMatchFileSnapshotSync: (
      filename?: string,
      options?: FileMatcherOptions,
    ) => void;
  }
}

declare global {
  type Expect = import('rstack/test').Expect;
  type Describe = import('rstack/test').Describe;
  type Assertion<T> = import('rstack/test').Assertion<T>;
}

export {};
