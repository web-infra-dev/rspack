import UiLib from 'ui-lib';
import { Button } from 'ui-lib-es';
import UiLibScopeSc from '@scope-sc/ui-lib';

export default () => {
  return `default Uilib has ${Object.keys(UiLib).join(
    ', ',
  )} exports not tree shaking, and ui-lib-es Button value is ${Button} should tree shaking`;
};

export const scopeScUILib = () => {
  return `scope-sc Uilib has ${Object.keys(UiLibScopeSc).join(
    ', ',
  )}`;
};

export const dynamicUISpecificExport = async () => {
  const { List } = await import('ui-lib-dynamic-specific-export');
  return `dynamic Uilib has ${List} exports tree shaking`;
};

export const dynamicUIDefaultExport = async () => {
  const uiLib = await import('ui-lib-dynamic-default-export');
  return `dynamic Uilib has ${uiLib.List} exports tree shaking`;
};

export const dynamicUISideEffectExport = async () => {
  const uiLibSideEffect = await import('ui-lib-side-effect');
  return `dynamic Uilib has ${uiLibSideEffect.List} exports not tree shaking`;
};
