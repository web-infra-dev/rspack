import { DemoTimepickerBasicComponent } from './demos/basic/basic';
import { DemoTimepickerMeridianComponent } from './demos/meridian/meridian';
import { DemoTimepickerCustomMeridianComponent } from './demos/custom-meridian/custom-meridian';
import { DemoTimepickerMinMaxComponent } from './demos/min-max/min-max';
import { DemoTimepickerPlaceholderComponent } from './demos/placeholder/placeholder';
import { DemoTimepickerToggleMinutesSecondsComponent } from './demos/toggle-minutes-seconds/toggle-minutes-seconds';
import { DemoTimepickerDisabledComponent } from './demos/disabled/disabled';
import { DemoTimepickerCustomComponent } from './demos/custom/custom';
import { DemoTimepickerCustomValidationComponent } from './demos/custom-validation/custom-validation';
import { DemoTimepickerDynamicComponent } from './demos/dynamic/dynamic';
import { DemoTimepickerMousewheelComponent } from './demos/mousewheel/mousewheel';
import { DemoTimepickerArrowkeysComponent } from './demos/arrowkeys/arrowkeys';
import { DemoTimepickerEmptyDateComponent } from './demos/empty-date/empty-date';
import { DemoTimepickerConfigComponent } from './demos/config/config';
import { DemoTimepickerReadonlyComponent } from './demos/readonly/readonly';
import { DemoTimepickerSpinnersComponent } from './demos/spinners/spinners';

import { ContentSection } from '../../common-docs';
import { ExamplesComponent } from '../../common-docs';
import { ApiSectionsComponent } from '../../common-docs';

import {
  NgApiDocComponent,
  NgApiDocConfigComponent
} from '../../common-docs';
import { DemoTimepickerIsValidComponent } from './demos/isvalid/isvalid';
import { DemoTimepickerFormComponent } from './demos/form/form';

export const demoComponentContent: ContentSection[] = [
  {
    name: 'Overview',
    anchor: 'overview',
    tabName: 'overview',
    outlet: ExamplesComponent,
    content: [
      {
        title: 'Basic',
        anchor: 'basic',
        component: require('!!raw-loader!./demos/basic/basic'),
        html: require('!!raw-loader!./demos/basic/basic.html'),
        outlet: DemoTimepickerBasicComponent
      },
      {
        title: 'Form',
        anchor: 'form',
        component: require('!!raw-loader!./demos/form/form'),
        html: require('!!raw-loader!./demos/form/form.html'),
        outlet: DemoTimepickerFormComponent
      },
      {
        title: 'Meridian',
        anchor: 'meridian',
        component: require('!!raw-loader!./demos/meridian/meridian'),
        html: require('!!raw-loader!./demos/meridian/meridian.html'),
        outlet: DemoTimepickerMeridianComponent
      },
      {
        title: 'Custom meridian',
        anchor: 'custom-meridian',
        component: require('!!raw-loader!./demos/custom-meridian/custom-meridian'),
        html: require('!!raw-loader!./demos/custom-meridian/custom-meridian.html'),
        description: `<p>Text in meridian labels can be customized by using <code>meridians</code> input property</p>`,
        outlet: DemoTimepickerCustomMeridianComponent
      },
      {
        title: 'Min - Max',
        anchor: 'min-max',
        component: require('!!raw-loader!./demos/min-max/min-max'),
        html: require('!!raw-loader!./demos/min-max/min-max.html'),
        outlet: DemoTimepickerMinMaxComponent
      },
      {
        title: 'Toggle minutes/seconds',
        anchor: 'toggleMinutesSeconds',
        component: require('!!raw-loader!./demos/toggle-minutes-seconds/toggle-minutes-seconds'),
        html: require('!!raw-loader!./demos/toggle-minutes-seconds/toggle-minutes-seconds.html'),
        outlet: DemoTimepickerToggleMinutesSecondsComponent
      },
      {
        title: 'Disabled',
        anchor: 'disabled',
        component: require('!!raw-loader!./demos/disabled/disabled'),
        html: require('!!raw-loader!./demos/disabled/disabled.html'),
        outlet: DemoTimepickerDisabledComponent
      },
      {
        title: 'Readonly',
        anchor: 'readonly',
        component: require('!!raw-loader!./demos/readonly/readonly'),
        html: require('!!raw-loader!./demos/readonly/readonly.html'),
        outlet: DemoTimepickerReadonlyComponent
      },
      {
        title: 'Custom steps',
        anchor: 'custom',
        component: require('!!raw-loader!./demos/custom/custom'),
        html: require('!!raw-loader!./demos/custom/custom.html'),
        outlet: DemoTimepickerCustomComponent
      },
      {
        title: 'Custom validation',
        anchor: 'custom-validation',
        component: require('!!raw-loader!./demos/custom-validation/custom-validation'),
        html: require('!!raw-loader!./demos/custom-validation/custom-validation.html'),
        outlet: DemoTimepickerCustomValidationComponent
      },
      {
        title: 'Custom validation with isValid event',
        anchor: 'isvalid',
        component: require('!!raw-loader!./demos/isvalid/isvalid'),
        html: require('!!raw-loader!./demos/isvalid/isvalid.html'),
        description: `<p><code>isValid</code> event emits true if a value is a valid data.
            Enter an invalid data to see error</p>`,
        outlet: DemoTimepickerIsValidComponent
      },
      {
        title: 'Dynamic',
        anchor: 'dynamic',
        component: require('!!raw-loader!./demos/dynamic/dynamic'),
        html: require('!!raw-loader!./demos/dynamic/dynamic.html'),
        outlet: DemoTimepickerDynamicComponent
      },
      {
        title: 'Mouse wheel',
        anchor: 'mouse-wheel',
        component: require('!!raw-loader!./demos/mousewheel/mousewheel'),
        html: require('!!raw-loader!./demos/mousewheel/mousewheel.html'),
        outlet: DemoTimepickerMousewheelComponent
      },
      {
        title: 'Empty date',
        anchor: 'empty-date',
        component: require('!!raw-loader!./demos/empty-date/empty-date'),
        html: require('!!raw-loader!./demos/empty-date/empty-date.html'),
        outlet: DemoTimepickerEmptyDateComponent
      },
      {
        title: 'Arrow keys',
        anchor: 'arrow keys',
        component: require('!!raw-loader!./demos/arrowkeys/arrowkeys'),
        html: require('!!raw-loader!./demos/arrowkeys/arrowkeys.html'),
        outlet: DemoTimepickerArrowkeysComponent
      },
      {
        title: 'Spinners',
        anchor: 'spinners',
        component: require('!!raw-loader!./demos/spinners/spinners'),
        html: require('!!raw-loader!./demos/spinners/spinners.html'),
        outlet: DemoTimepickerSpinnersComponent
      },
      {
        title: 'Placeholder',
        anchor: 'placeholder',
        component: require('!!raw-loader!./demos/placeholder/placeholder'),
        html: require('!!raw-loader!./demos/placeholder/placeholder.html'),
        outlet: DemoTimepickerPlaceholderComponent
      },
      {
        title: 'Configuring defaults',
        anchor: 'config-defaults',
        component: require('!!raw-loader!./demos/config/config'),
        html: require('!!raw-loader!./demos/config/config.html'),
        outlet: DemoTimepickerConfigComponent
      }
    ]
  },
  {
    name: 'Installation',
    anchor: 'api-reference',
    tabName: 'api',
    outlet: ApiSectionsComponent,
    usage: require('!!raw-loader!./docs/usage.md'),
    importInfo: '<span class="pln">ng add ngx</span><span class="pun">-</span><span class="pln">bootstrap </span> --component <span class="pln">timepicker</span>',
    content: [
      {
        title: 'TimepickerComponent',
        anchor: 'timepicker-component',
        outlet: NgApiDocComponent
      },
      {
        title: 'TimepickerConfig',
        anchor: 'timepicker-config',
        outlet: NgApiDocConfigComponent
      }
    ]
  },
  {
    name: 'Examples',
    anchor: 'examples',
    tabName: 'examples',
    outlet: ExamplesComponent,
    content: [
      {
        title: 'Basic',
        anchor: 'basic-ex',
        outlet: DemoTimepickerBasicComponent
      },
      {
        title: 'Form',
        anchor: 'form-ex',
        outlet: DemoTimepickerFormComponent
      },
      {
        title: 'Meridian',
        anchor: 'meridian-ex',
        outlet: DemoTimepickerMeridianComponent
      },
      {
        title: 'Custom meridian',
        anchor: 'custom-meridian-ex',
        outlet: DemoTimepickerCustomMeridianComponent
      },
      {
        title: 'Min - Max',
        anchor: 'min-max-ex',
        outlet: DemoTimepickerMinMaxComponent
      },
      {
        title: 'Toggle minutes/seconds',
        anchor: 'toggleMinutesSeconds-ex',
        outlet: DemoTimepickerToggleMinutesSecondsComponent
      },
      {
        title: 'Disabled',
        anchor: 'disabled-ex',
        outlet: DemoTimepickerDisabledComponent
      },
      {
        title: 'Readonly',
        anchor: 'readonly-ex',
        outlet: DemoTimepickerReadonlyComponent
      },
      {
        title: 'Custom steps',
        anchor: 'custom-ex',
        outlet: DemoTimepickerCustomComponent
      },
      {
        title: 'Custom validation',
        anchor: 'custom-validation-ex',
        outlet: DemoTimepickerCustomValidationComponent
      },
      {
        title: 'Custom validation with isValid event',
        anchor: 'isvalid-ex',
        outlet: DemoTimepickerIsValidComponent
      },
      {
        title: 'Dynamic',
        anchor: 'dynamic-ex',
        outlet: DemoTimepickerDynamicComponent
      },
      {
        title: 'Mouse wheel',
        anchor: 'mouse-wheel-ex',
        outlet: DemoTimepickerMousewheelComponent
      },
      {
        title: 'Empty date',
        anchor: 'empty-date-ex',
        outlet: DemoTimepickerEmptyDateComponent
      },
      {
        title: 'Arrow keys',
        anchor: 'arrow keys-ex',
        outlet: DemoTimepickerArrowkeysComponent
      },
      {
        title: 'Spinners',
        anchor: 'spinners-ex',
        outlet: DemoTimepickerSpinnersComponent
      },
      {
        title: 'Placeholder',
        anchor: 'placeholder-ex',
        outlet: DemoTimepickerPlaceholderComponent
      },
      {
        title: 'Configuring defaults',
        anchor: 'config-defaults-ex',
        outlet: DemoTimepickerConfigComponent
      }
    ]
  }
];
