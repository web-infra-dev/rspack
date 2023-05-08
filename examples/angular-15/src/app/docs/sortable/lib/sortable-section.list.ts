import { DemoBasicComponent } from './demos/basic/basic.component';
import { ComplexDatamodelDemoComponent } from './demos/complex-datamodel/complex-datamodel.component';
import { CustomItemTemplateDemoComponent } from './demos/custom-item-template/custom-item-template';

import { ContentSection } from '../../common-docs';
import { ExamplesComponent } from '../../common-docs';
import { ApiSectionsComponent } from '../../common-docs';

import { NgApiDocComponent } from '../../common-docs';
import { DemoAccessibilityComponent } from './demos/accessibility/accessibility';

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
        component: require('!!raw-loader!./demos/basic/basic.component.ts'),
        html: require('!!raw-loader!./demos/basic/basic.component.html'),
        outlet: DemoBasicComponent
      },
      {
        title: 'Complex data model',
        anchor: 'complexDatamodel',
        component: require('!!raw-loader!./demos/complex-datamodel/complex-datamodel.component.ts'),
        html: require('!!raw-loader!./demos/complex-datamodel/complex-datamodel.component.html'),
        outlet: ComplexDatamodelDemoComponent
      },
      {
        title: 'Custom item template',
        anchor: 'itemTemplate',
        component: require('!!raw-loader!./demos/custom-item-template/custom-item-template.ts'),
        html: require('!!raw-loader!./demos/custom-item-template/custom-item-template.html'),
        outlet: CustomItemTemplateDemoComponent
      },
      {
        title: 'Accessibility',
        anchor: 'accessibility',
        outlet: DemoAccessibilityComponent
      }
    ]
  },
  {
    name: 'Installation',
    anchor: 'api-reference',
    tabName: 'api',
    outlet: ApiSectionsComponent,
    usage: require('!!raw-loader!./docs/usage.md'),
    importInfo: '<span class="pln">ng add ngx</span><span class="pun">-</span><span class="pln">bootstrap </span> --component <span class="pln">sortable</span>',
    content: [
      {
        title: 'SortableComponent',
        anchor: 'sortable-component',
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
        outlet: DemoBasicComponent
      },
      {
        title: 'Complex data model',
        anchor: 'complexDatamodel-ex',
        outlet: ComplexDatamodelDemoComponent
      },
      {
        title: 'Custom item template',
        anchor: 'itemTemplate-ex',
        outlet: CustomItemTemplateDemoComponent
      }
    ]
  }
];
