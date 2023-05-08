import { DemoButtonsBasicComponent } from './demos/basic/basic';
import { DemoButtonsCheckboxComponent } from './demos/checkbox/checkbox';
import { DemoButtonsRadioComponent } from './demos/radio/radio';
import { DemoButtonsUncheckableRadioComponent } from './demos/uncheckable-radio/uncheckable-radio';
import { DemoButtonsCheckboxReactiveFormsComponent } from './demos/checkbox-reactiveforms/checkbox-reactiveforms';
import { DemoButtonsRadioReactiveFormsComponent } from './demos/radio-reactiveforms/radio-reactiveforms';
import { DemoButtonsDisabledComponent } from './demos/disabled/disabled';
import { DemoButtonsCustomCheckboxValueComponent } from './demos/custom-checkbox-value/custom-checkbox-value';
import { DemoButtonsRadioWithGroupComponent } from './demos/radio-with-group/radio-with-group';

import { ContentSection } from '../../common-docs';
import { ExamplesComponent } from '../../common-docs';
import { ApiSectionsComponent } from '../../common-docs';

import { NgApiDocComponent } from '../../common-docs';

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
        component: require('!!raw-loader!./demos/basic/basic.ts'),
        html: require('!!raw-loader!./demos/basic/basic.html'),
        outlet: DemoButtonsBasicComponent
      },
      {
        title: 'Checkbox',
        anchor: 'checkbox',
        description: `Checkbox-like buttons set with variable states`,
        component: require('!!raw-loader!./demos/checkbox/checkbox.ts'),
        html: require('!!raw-loader!./demos/checkbox/checkbox.html'),
        outlet: DemoButtonsCheckboxComponent
      },
      {
        title: 'Custom checkbox value',
        anchor: 'custom-checkbox-value',
        component: require('!!raw-loader!./demos/custom-checkbox-value/custom-checkbox-value.ts'),
        html: require('!!raw-loader!./demos/custom-checkbox-value/custom-checkbox-value.html'),
        outlet: DemoButtonsCustomCheckboxValueComponent
      },
      {
        title: 'Checkbox with Reactive Forms',
        anchor: 'checkbox-reactiveforms',
        description: `Checkbox buttons with ReactiveForms`,
        component: require('!!raw-loader!./demos/checkbox-reactiveforms/checkbox-reactiveforms.ts'),
        html: require('!!raw-loader!./demos/checkbox-reactiveforms/checkbox-reactiveforms.html'),
        outlet: DemoButtonsCheckboxReactiveFormsComponent
      },
      {
        title: 'Radio with radio group',
        anchor: 'radio-button-with-group',
        description: `Radio buttons with checked/unchecked states. Radio buttons used together with a <code>btnRadioGroup</code> can be
used in template driven and reactive forms.
They follow the <a href="https://www.w3.org/TR/wai-aria-practices-1.1/#radiobutton">W3C WAI-AIRA design pattern for radio groups</a>.
Meaning
<ul>
<li>The Radio Group is inserted in the tab-order of the page by automatically adding a tabindex attribute</li>
<li>The selected radio element can be changed with the arrow keys if the focus is in the group</li>
<li>The role of the group is set to "radiogroup" and the aria-checked attributes are added according to the state</li>
</ul>
Individual buttons or the whole group can be marked as disabled.
`,
        component: require('!!raw-loader!./demos/radio-with-group/radio-with-group.ts'),
        html: require('!!raw-loader!./demos/radio-with-group/radio-with-group.html'),
        outlet: DemoButtonsRadioWithGroupComponent
      },
      {
        title: 'Radio without explicit group',
        anchor: 'radio-button-explicit-group',
        description: ` The second method to create a radio button group is to use the same <code>ngModel</code> binding with several buttons.
 This works only for template driven forms and is not generally advised. But there are use cases were this might be useful, e.g. in tables.
 In terms of accessibility the buttons in the group can not be selected with the arrow keys, but individually reached by using the tab key
 and then be toggled by using the space key. You can check out the demo below.`,
        component: require('!!raw-loader!./demos/radio/radio.ts'),
        html: require('!!raw-loader!./demos/radio/radio.html'),
        outlet: DemoButtonsRadioComponent
      },
      {
        title: 'Uncheckable Radio',
        anchor: 'uncheckable-radio-button',
        component: require('!!raw-loader!./demos/uncheckable-radio/uncheckable-radio.ts'),
        html: require('!!raw-loader!./demos/uncheckable-radio/uncheckable-radio.html'),
        outlet: DemoButtonsUncheckableRadioComponent
      },
      {
        title: 'Radio with Reactive Forms',
        anchor: 'radio-reactiveforms',
        description: `Radio buttons with ReactiveForms. Example below shows how to use radio buttons with reactive
 forms. Please be aware that for reactive forms it's required to use <code>btnRadioGroup</code> directive along with
 <code>btnRadio</code>'s`,
        component: require('!!raw-loader!./demos/radio-reactiveforms/radio-reactiveforms.ts'),
        html: require('!!raw-loader!./demos/radio-reactiveforms/radio-reactiveforms.html'),
        outlet: DemoButtonsRadioReactiveFormsComponent
      },
      {
        title: 'Disabled Buttons',
        anchor: 'disabled-buttons',
        component: require('!!raw-loader!./demos/disabled/disabled.ts'),
        html: require('!!raw-loader!./demos/disabled/disabled.html'),
        outlet: DemoButtonsDisabledComponent
      }
    ]
  },
  {
    name: 'Installation',
    anchor: 'api-reference',
    tabName: 'api',
    outlet: ApiSectionsComponent,
    usage: require('!!raw-loader!./docs/usage.md'),
    importInfo: '<span class="pln">ng add ngx</span><span class="pun">-</span><span class="pln">bootstrap </span> &#45;&#45;component <span class="pln">buttons</span>',
    content: [
      {
        title: 'ButtonCheckboxDirective',
        anchor: 'button-checkbox-directive',
        outlet: NgApiDocComponent
      },
      {
        title: 'ButtonRadioDirective',
        anchor: 'button-radio-directive',
        outlet: NgApiDocComponent
      },
      {
        title: 'ButtonRadioGroupDirective',
        anchor: 'button-radio-group-directive',
        outlet: NgApiDocComponent
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
        outlet: DemoButtonsBasicComponent
      },
      {
        title: 'Checkbox',
        anchor: 'checkbox-ex',
        outlet: DemoButtonsCheckboxComponent
      },
      {
        title: 'Custom checkbox value',
        anchor: 'custom-checkbox-value-ex',
        outlet: DemoButtonsCustomCheckboxValueComponent
      },
      {
        title: 'Checkbox with Reactive Forms',
        anchor: 'checkbox-reactiveforms-ex',
        outlet: DemoButtonsCheckboxReactiveFormsComponent
      },
      {
        title: 'Radio with radio group',
        anchor: 'radio-button-with-group-ex',
        outlet: DemoButtonsRadioWithGroupComponent
      },
      {
        title: 'Radio without explicit group',
        anchor: 'radio-button-explicit-group-ex',
        outlet: DemoButtonsRadioComponent
      },
      {
        title: 'Uncheckable Radio',
        anchor: 'uncheckable-radio-button-ex',
        outlet: DemoButtonsUncheckableRadioComponent
      },
      {
        title: 'Radio with Reactive Forms',
        anchor: 'radio-reactiveforms-ex',
        outlet: DemoButtonsRadioReactiveFormsComponent
      },
      {
        title: 'Disabled Buttons',
        anchor: 'disabled-buttons-ex',
        outlet: DemoButtonsDisabledComponent
      }
    ]
  }
];
