import { DemoAccordionAnimatedComponent } from './demos/animated/animated';
import { DemoAccordionBasicComponent } from './demos/basic/basic';
import { DemoAccordionConfigComponent } from './demos/config/config';
import { DemoAccordionCustomHTMLComponent } from './demos/custom-html/custom-html';
import { DemoAccordionDisabledComponent } from './demos/disabled/disabled';
import { DemoAccordionDynamicComponent } from './demos/dymanic/dynamic';
import { DemoAccordionManualToggleComponent } from './demos/manual-toggle/manual-toggle';
import { DemoAccordionOneAtATimeComponent } from './demos/one-at-a-time/one-at-a-time';
import { DemoAccordionOpenEventComponent } from './demos/open-event/open-event';
import { DemoAccordionStylingComponent } from './demos/styling/styling';

import { ContentSection } from '../../common-docs';

import { ExamplesComponent } from '../../common-docs';
import { ApiSectionsComponent } from '../../common-docs';

import {
  NgApiDocComponent,
  NgApiDocConfigComponent
} from '../../common-docs';
import { DemoAccordionOpenedComponent } from './demos/opened/opened';
import { DemoAccordionDynamicBodyComponent } from './demos/dynamic-body/dynamic-body';

export const demoComponentContent: ContentSection[] = [
  {
    name: 'Overview',
    anchor: 'overview',
    tabName: 'overview',
    outlet: ExamplesComponent,
    content: [
      {
        title: 'Basic',
        anchor: 'basic-accordion',
        description: `<p>Click headers to expand/collapse content that is broken into logical sections, much
          like tabs.</p>`,
        component: require('!!raw-loader!./demos/basic/basic'),
        html: require('!!raw-loader!./demos/basic/basic.html'),
        outlet: DemoAccordionBasicComponent
      },
      {
        title: 'With animation',
        anchor: 'animated-accordion',
        description: `<p>Use input property or config property <code>isAnimated</code> to enable/disable animation</p>`,
        component: require('!!raw-loader!./demos/animated/animated'),
        html: require('!!raw-loader!./demos/animated/animated.html'),
        outlet: DemoAccordionAnimatedComponent
      },
      {
        title: 'Group opening event',
        anchor: 'open-event',
        description: `<p>Accordion with <code>isOpenChange</code> event listener.</p>`,
        component: require('!!raw-loader!./demos/open-event/open-event'),
        html: require('!!raw-loader!./demos/open-event/open-event.html'),
        outlet: DemoAccordionOpenEventComponent
      },
      {
        title: 'Custom HTML',
        anchor: 'custom-html',
        component: require('!!raw-loader!./demos/custom-html/custom-html'),
        html: require('!!raw-loader!./demos/custom-html/custom-html.html'),
        outlet: DemoAccordionCustomHTMLComponent
      },
      {
        title: 'Disabled',
        anchor: 'disabled',
        component: require('!!raw-loader!./demos/disabled/disabled'),
        html: require('!!raw-loader!./demos/disabled/disabled.html'),
        outlet: DemoAccordionDisabledComponent
      },
      {
        title: 'Initially opened',
        anchor: 'opened',
        component: require('!!raw-loader!./demos/opened/opened'),
        html: require('!!raw-loader!./demos/opened/opened.html'),
        outlet: DemoAccordionOpenedComponent
      },
      {
        title: 'Dynamic accordion',
        anchor: 'dynamic-accordion',
        component: require('!!raw-loader!./demos/dymanic/dynamic'),
        html: require('!!raw-loader!./demos/dymanic/dynamic.html'),
        outlet: DemoAccordionDynamicComponent
      },
      {
        title: 'Dynamic body content',
        anchor: 'dynamic-body',
        component: require('!!raw-loader!./demos/dynamic-body/dynamic-body'),
        html: require('!!raw-loader!./demos/dynamic-body/dynamic-body.html'),
        outlet: DemoAccordionDynamicBodyComponent
      },
      {
        title: 'Manual toggle',
        anchor: 'manual-toggle',
        component: require('!!raw-loader!./demos/manual-toggle/manual-toggle'),
        html: require('!!raw-loader!./demos/manual-toggle/manual-toggle.html'),
        outlet: DemoAccordionManualToggleComponent
      },
      {
        title: 'Open only one at a time',
        anchor: 'one-time',
        component: require('!!raw-loader!./demos/one-at-a-time/one-at-a-time'),
        html: require('!!raw-loader!./demos/one-at-a-time/one-at-a-time.html'),
        outlet: DemoAccordionOneAtATimeComponent
      },
      {
        title: 'Styling',
        anchor: 'styling',
        component: require('!!raw-loader!./demos/styling/styling'),
        html: require('!!raw-loader!./demos/styling/styling.html'),
        outlet: DemoAccordionStylingComponent
      },
      {
        title: 'Configuring defaults',
        anchor: 'config',
        component: require('!!raw-loader!./demos/config/config'),
        html: require('!!raw-loader!./demos/config/config.html'),
        outlet: DemoAccordionConfigComponent
      }
    ]
  },
  {
    name: 'API Reference',
    anchor: 'api-reference',
    tabName: 'api',
    importInfo: '<span class="pln">ng add ngx</span><span class="pun">-</span><span class="pln">bootstrap </span> --component <span class="pln">accordion</span>',
    usage: require('!!raw-loader!./docs/usage.md'),
    outlet: ApiSectionsComponent,
    content: [
      {
        title: 'AccordionComponent',
        anchor: 'AccordionComponent',
        outlet: NgApiDocComponent
      },
      {
        title: 'AccordionPanelComponent',
        anchor: 'AccordionPanelComponent',
        outlet: NgApiDocComponent
      },
      {
        title: 'AccordionConfig',
        anchor: 'AccordionConfig',
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
        anchor: 'basic-accordion-ex',
        outlet: DemoAccordionBasicComponent
      },
      {
        title: 'With animation',
        anchor: 'animated-accordion-ex',
        outlet: DemoAccordionAnimatedComponent
      },
      {
        title: 'Group opening event',
        anchor: 'open-event-ex',
        outlet: DemoAccordionOpenEventComponent
      },
      {
        title: 'Custom HTML',
        anchor: 'custom-html-ex',
        outlet: DemoAccordionCustomHTMLComponent
      },
      {
        title: 'Disabled',
        anchor: 'disabled-ex',
        outlet: DemoAccordionDisabledComponent
      },
      {
        title: 'Initially opened',
        anchor: 'opened-ex',
        outlet: DemoAccordionOpenedComponent
      },
      {
        title: 'Dynamic accordion',
        anchor: 'dynamic-accordion-ex',
        outlet: DemoAccordionDynamicComponent
      },
      {
        title: 'Dynamic body content',
        anchor: 'dynamic-body-ex',
        outlet: DemoAccordionDynamicBodyComponent
      },
      {
        title: 'Manual toggle',
        anchor: 'manual-toggle-ex',
        outlet: DemoAccordionManualToggleComponent
      },
      {
        title: 'Open only one at a time',
        anchor: 'one-time-ex',
        outlet: DemoAccordionOneAtATimeComponent
      },
      {
        title: 'Styling',
        anchor: 'styling-ex',
        outlet: DemoAccordionStylingComponent
      },
      {
        title: 'Configuring defaults',
        anchor: 'config-ex',
        outlet: DemoAccordionConfigComponent
      }
    ]
  }
];
