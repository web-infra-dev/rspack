export async function loadSecondFeature() {
  const { loadSecondFeature } = await import(
    /* webpackChunkName: "shared" */ './shared'
  );
  return loadSecondFeature();
}
