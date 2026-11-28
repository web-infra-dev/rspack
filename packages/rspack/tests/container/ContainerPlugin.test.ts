import { container } from '@rspack/core';
import { describe, expect, it } from 'rstack/test';

const { ContainerPlugin, ContainerReferencePlugin } = container;

it.each([undefined, null, 123])(
  'rejects a non-string container name %s',
  (name) => {
    // @ts-expect-error Exercise invalid names supplied by JavaScript callers.
    expect(() => new ContainerPlugin({ name, exposes: {} })).toThrow(
      '[ContainerPlugin] name must be a string.',
    );
  },
);

describe.each([false, true])('shareScope with enhanced=%s', (enhanced) => {
  it('rejects an empty container share scope', () => {
    expect(
      () =>
        new ContainerPlugin({
          name: 'container',
          exposes: {},
          shareScope: [],
          enhanced,
        }),
    ).toThrow(
      '[ContainerPlugin] shareScope array must contain at least one entry',
    );
  });

  it('rejects an empty container reference share scope', () => {
    expect(
      () =>
        new ContainerReferencePlugin({
          remoteType: 'commonjs',
          remotes: {},
          shareScope: [],
          enhanced,
        }),
    ).toThrow(
      '[ContainerReferencePlugin] shareScope array must contain at least one entry',
    );
  });

  it('rejects an empty share scope on an individual remote', () => {
    expect(
      () =>
        new ContainerReferencePlugin({
          remoteType: 'commonjs',
          remotes: { remote: { external: './remote', shareScope: [] } },
          enhanced,
        }),
    ).toThrow(
      '[ContainerReferencePlugin] shareScope array must contain at least one entry',
    );
  });
});

it('rejects multiple container share scopes without the enhanced runtime', () => {
  expect(
    () =>
      new ContainerPlugin({
        name: 'container',
        exposes: {},
        shareScope: ['custom', 'other'],
      }),
  ).toThrow(
    'shareScope as an array with multiple entries requires enhanced=true',
  );
});
