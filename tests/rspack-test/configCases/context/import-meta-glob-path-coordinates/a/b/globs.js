export const parentGlob = import.meta.glob('../*/*.mjs', {
  eager: true,
  import: 'default',
});

export const parentExclusion = import.meta.glob(['./**/*.mjs', '!../**/*.mjs'], {
  eager: true,
  import: 'default',
});

export const outsideGlob = import.meta.glob('../../outside/*.js', {
  eager: true,
  import: 'default',
});

export const bracedExclusion = import.meta.glob(
  ['../../outside/*.js', '!../{../outside/*.js,inside/*.js}'],
  { eager: true, import: 'default' },
);
