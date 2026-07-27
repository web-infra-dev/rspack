/// <reference types="@rspack/core/import-meta-hot" />

if (import.meta.hot) {
  import.meta.hot.accept();
  import.meta.hot.accept((mod) => {
    mod?.default;
  });
  import.meta.hot.accept('./dep', (mod) => {
    mod?.default;
  });
  import.meta.hot.accept(['./a', './b'] as const, (mods) => {
    mods[0]?.default;
    mods[1]?.default;
  });
  import.meta.hot.dispose((data) => {
    data.disposed = true;
  });

  // @ts-expect-error webpack-only API
  import.meta.hot.decline();
  // @ts-expect-error transport API is not implemented by this runtime
  import.meta.hot.send('event');
}
