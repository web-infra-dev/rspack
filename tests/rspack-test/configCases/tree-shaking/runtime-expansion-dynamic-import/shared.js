export const live = 'live';

export const loadFeature = () =>
  import(/* webpackChunkName: "leaf" */ './leaf');

export const loadSecondFeature = () =>
  import(/* webpackChunkName: "second-leaf" */ './second-leaf');
