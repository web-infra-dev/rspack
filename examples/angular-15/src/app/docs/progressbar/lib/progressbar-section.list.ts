import { DemoProgressbarStaticComponent } from './demos/static/static';
import { DemoProgressbarDynamicComponent } from './demos/dynamic/dynamic';
import { DemoProgressbarStackedComponent } from './demos/stacked/stacked';
import { DemoProgressbarConfigComponent } from './demos/config/config';

import { ContentSection } from '../../common-docs';
import { ExamplesComponent } from '../../common-docs';
import { ApiSectionsComponent } from '../../common-docs';

import {
  NgApiDocComponent,
  NgApiDocConfigComponent
} from '../../common-docs';

export const demoComponentContent: ContentSection[] = [
  {
    name: 'Overview',
    anchor: 'overview',
    tabName: 'overview',
    outlet: ExamplesComponent,
    content: [
      {
        title: 'Static',
        anchor: 'bs3-static',
        component: require('!!raw-loader!./demos/static/static'),
        html: require('!!raw-loader!./demos/static/static.html'),
        outlet: DemoProgressbarStaticComponent
      },
      {
        title: 'Dynamic',
        anchor: 'bs3-dynamic',
        component: require('!!raw-loader!./demos/dynamic/dynamic'),
        html: require('!!raw-loader!./demos/dynamic/dynamic.html'),
        outlet: DemoProgressbarDynamicComponent
      },
      {
        title: 'Stacked',
        anchor: 'bs3-stacked',
        component: require('!!raw-loader!./demos/stacked/stacked'),
        html: require('!!raw-loader!./demos/stacked/stacked.html'),
        outlet: DemoProgressbarStackedComponent
      },
      {
        title: 'Configuring defaults',
        anchor: 'bs3-config',
        component: require('!!raw-loader!./demos/config/config'),
        html: require('!!raw-loader!./demos/config/config.html'),
        outlet: DemoProgressbarConfigComponent
      }
    ]
  },
  {
    name: 'Installation',
    anchor: 'api-reference',
    tabName: 'api',
    outlet: ApiSectionsComponent,
    usage: require('!!raw-loader!./docs/usage.md'),
    importInfo: '<span class="pln">ng add ngx</span><span class="pun">-</span><span class="pln">bootstrap </span> --component <span class="pln">progressbar</span>',
    content: [
      {
        title: 'ProgressbarComponent',
        anchor: 'progressbar-component',
        outlet: NgApiDocComponent
      },
      {
        title: 'ProgressbarConfig',
        anchor: 'progress-config',
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
        title: 'Static',
        anchor: 'bs3-static-ex',
        outlet: DemoProgressbarStaticComponent
      },
      {
        title: 'Dynamic',
        anchor: 'bs3-dynamic-ex',
        outlet: DemoProgressbarDynamicComponent
      },
      {
        title: 'Stacked',
        anchor: 'bs3-stacked-ex',
        outlet: DemoProgressbarStackedComponent
      },
      {
        title: 'Configuring defaults',
        anchor: 'bs3-config-ex',
        outlet: DemoProgressbarConfigComponent
      }
    ]
  }
];

