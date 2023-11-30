import { createGlobalTheme } from '@vanilla-extract/css';
import { modularScale } from 'polished';
import colors from 'tailwindcss/colors';

const createScale = (ratio: number, base: number) => (steps: number) =>
  `${modularScale(steps, base, ratio)}px`;

const spaceScale = createScale(1.4, 4);
const fontSizeScale = createScale(1.3, 16);
const lineHeightScale = createScale(1.25, 24);
const borderRadiusScale = createScale(1.5, 4);

export const vars = createGlobalTheme(':root', {
  space: {
    none: '0',
    '0x': spaceScale(0),
    '1x': spaceScale(1),
    '2x': spaceScale(2),
    '3x': spaceScale(3),
    '4x': spaceScale(4),
    '5x': spaceScale(5),
    '6x': spaceScale(6),
    '7x': spaceScale(7),
    '8x': spaceScale(8),
  },
  color: {
    white: '#fff',

    'gray-50': colors.coolGray[50],
    'gray-100': colors.coolGray[100],
    'gray-200': colors.coolGray[200],
    'gray-300': colors.coolGray[300],
    'gray-400': colors.coolGray[400],
    'gray-500': colors.coolGray[500],
    'gray-600': colors.coolGray[600],
    'gray-700': colors.coolGray[700],
    'gray-800': colors.coolGray[800],
    'gray-900': colors.coolGray[900],

    'green-50': colors.emerald[50],
    'green-100': colors.emerald[100],
    'green-200': colors.emerald[200],
    'green-300': colors.emerald[300],
    'green-400': colors.emerald[400],
    'green-500': colors.emerald[500],
    'green-600': colors.emerald[600],
    'green-700': colors.emerald[700],
    'green-800': colors.emerald[800],
    'green-900': colors.emerald[900],
  },
  borderRadius: {
    '0x': borderRadiusScale(0),
    '1x': borderRadiusScale(1),
    '2x': borderRadiusScale(2),
    '3x': borderRadiusScale(3),
    '4x': borderRadiusScale(4),
    '5x': borderRadiusScale(5),
    full: '99999px',
  },
  fontFamily: {
    body: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif, "Apple Color Emoji", "Segoe UI Emoji", "Segoe UI Symbol"',
  },
  fontSize: {
    '0x': fontSizeScale(0),
    '1x': fontSizeScale(1),
    '2x': fontSizeScale(2),
    '3x': fontSizeScale(3),
    '4x': fontSizeScale(4),
    '5x': fontSizeScale(5),
  },
  lineHeight: {
    '0x': lineHeightScale(0),
    '1x': lineHeightScale(1),
    '2x': lineHeightScale(2),
    '3x': lineHeightScale(3),
    '4x': lineHeightScale(4),
    '5x': lineHeightScale(5),
  },
});
