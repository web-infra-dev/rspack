import { DemoRatingBasicComponent } from './demos/basic/basic';
import { DemoRatingCustomComponent } from './demos/custom/custom';
import { DemoRatingDynamicComponent } from './demos/dynamic/dynamic';
import { DemoRatingSelectOnEnterComponent } from './demos/select-on-enter/select-on-enter';
import { DemoRatingConfigComponent } from './demos/config/config';

import { ContentSection } from '../../common-docs';
import { ExamplesComponent } from '../../common-docs';
import { ApiSectionsComponent } from '../../common-docs';

import { NgApiDocComponent, NgApiDocConfigComponent } from '../../common-docs';

export const demoComponentContent: ContentSection[] = [
  {
    name: 'Overview',
    anchor: 'overview',
    tabName: 'overview',
    outlet: ExamplesComponent,
    content: [
      {
        title: 'Basic rating',
        anchor: 'rating-basic',
        component: require('!!raw-loader!./demos/basic/basic'),
        html: require('!!raw-loader!./demos/basic/basic.html'),
        outlet: DemoRatingBasicComponent
      },
      {
        title: 'Dynamic rating',
        anchor: 'rating-dynamic',
        component: require('!!raw-loader!./demos/dynamic/dynamic'),
        html: require('!!raw-loader!./demos/dynamic/dynamic.html'),
        outlet: DemoRatingDynamicComponent
      },
      {
        title: 'Custom icons',
        anchor: 'rating-custom',
        component: require('!!raw-loader!./demos/custom/custom'),
        html: require('!!raw-loader!./demos/custom/custom.html'),
        outlet: DemoRatingCustomComponent
      },
      {
        title: 'Select on enter',
        description: `Key navigation example. Focus on rating and use arrow keys to set its value,
          then press <code>Enter</code> to select the value, after this, the rating state will be changed to readonly.`,
        anchor: 'select-on-enter',
        component: require('!!raw-loader!./demos/select-on-enter/select-on-enter'),
        html: require('!!raw-loader!./demos/select-on-enter/select-on-enter.html'),
        outlet: DemoRatingSelectOnEnterComponent
      },
      {
        title: 'Configuring defaults',
        anchor: 'demo-rating-config',
        component: require('!!raw-loader!./demos/config/config'),
        html: require('!!raw-loader!./demos/config/config.html'),
        outlet: DemoRatingConfigComponent
      }
    ]
  },
  {
    name: 'Installation',
    anchor: 'api-reference',
    tabName: 'api',
    usage: require('!!raw-loader!./docs/usage.md'),
    importInfo: '<span class="pln">ng add ngx</span><span class="pun">-</span><span class="pln">bootstrap </span> --component <span class="pln">rating</span>',
    outlet: ApiSectionsComponent,
    content: [
      {
        title: 'RatingComponent',
        anchor: 'rating-component',
        outlet: NgApiDocComponent
      },
      {
        title: 'RatingConfig',
        anchor: 'rating-config',
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
        title: 'Basic rating',
        anchor: 'rating-basic-ex',
        outlet: DemoRatingBasicComponent
      },
      {
        title: 'Dynamic rating',
        anchor: 'rating-dynamic-ex',
        outlet: DemoRatingDynamicComponent
      },
      {
        title: 'Custom icons',
        anchor: 'rating-custom-ex',
        outlet: DemoRatingCustomComponent
      },
      {
        title: 'Select on enter',
        anchor: 'select-on-enter-ex',
        outlet: DemoRatingSelectOnEnterComponent
      },
      {
        title: 'Configuring defaults',
        anchor: 'demo-rating-config-ex',
        outlet: DemoRatingConfigComponent
      }
    ]
  }
];
