import { style } from '@vanilla-extract/css';
import { sprinkles } from './sprinkles.css';

export const card = style([
  sprinkles({
    background: {
      lightMode: 'green-50',
      darkMode: 'gray-800',
    },
    borderRadius: {
      mobile: '4x',
      desktop: '5x',
    },
    padding: {
      mobile: '7x',
      desktop: '8x',
    },
  }),
  {
    transition: 'transform 4s ease-in-out',
    ':hover': {
      cursor: 'default',
      transform: 'scale(2) rotate(720deg)',
    },
  },
]);
