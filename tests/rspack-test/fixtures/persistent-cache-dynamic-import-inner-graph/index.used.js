import { live, loadEagerFeature, loadFeature } from './module';

console.log(live);
loadFeature().then(({ value }) => console.log(value));
loadEagerFeature().then(({ value }) => console.log(value));
