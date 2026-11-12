let style = {};

export function hasAnimation(element) {
  ({ style } = element);
  return 'animation' in style;
}
