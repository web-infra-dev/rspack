import UiLib1, { setVersion as setVersion1 } from '@scope-sc/ui-lib';
import UiLib2, { setVersion as setVersion2 } from '@scope-sc/ui-lib2';
import UiLib3, { setVersion as setVersion3 } from '@scope-sc/ui-lib3';

export default () => {
  setVersion1('0.1.3');
  setVersion2('0.1.4');
  setVersion3('0.1.5');
  return `UiLib1: ${UiLib1()}
  UiLib2: ${UiLib2()}
  UiLib3: ${UiLib3()}
  `;
};
