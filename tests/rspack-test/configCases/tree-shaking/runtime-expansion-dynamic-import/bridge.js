export async function loadFeature() {
  const { loadFeature } = await import(
    /* webpackChunkName: "shared" */ './shared'
  );
  return loadFeature();
}
