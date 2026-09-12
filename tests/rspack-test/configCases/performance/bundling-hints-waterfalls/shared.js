export default async () =>
  (await import(/* webpackChunkName: 'leaf' */ './leaf')).default;
