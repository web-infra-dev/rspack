import { ApiSectionsComponent } from '../../common-docs';
import { CollapseDemoAnimatedComponent } from './demos/animated/animated';
import { CollapseDemoComponent } from './demos/basic/basic';
import { CollapseDemoEventsComponent } from './demos/events/events';
import { ContentSection } from '../../common-docs';
import { DemoAccessibilityComponent } from './demos/accessibility/accessibility';
import { ExamplesComponent } from '../../common-docs';
import { InlineDisplayDemoComponent } from './demos/inline-display/inline-display';
import { ToggleManualDemoComponent } from './demos/toggle-manual/toggle-manual';

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
        component: require('!!raw-loader!./demos/basic/basic'),
        html: require('!!raw-loader!./demos/basic/basic.html'),
        outlet: CollapseDemoComponent
      },
      {
        title: 'With animation',
        anchor: 'animated',
        component: require('!!raw-loader!./demos/animated/animated'),
        html: require('!!raw-loader!./demos/animated/animated.html'),
        description: `You can enable animation via <code>isAnimated</code> input option`,
        outlet: CollapseDemoAnimatedComponent
      },
      {
        title: 'Events',
        anchor: 'events',
        component: require('!!raw-loader!./demos/events/events'),
        html: require('!!raw-loader!./demos/events/events.html'),
        description: `Collapse directive exposes 4 events: <code>collapses</code>, that fires when a collapse was triggered (animation start),
                        <code>collapsed</code>, that fires when a content was hidden (animation finished),
                        <code>expands</code>, that fires when a expansion was triggered (animation start)
                      and <code>expanded</code>, that fires when a content was shown`,
        outlet: CollapseDemoEventsComponent
      },
      {
        title: 'Manual toggle',
        anchor: 'manual-toggle',
        component: require('!!raw-loader!./demos/toggle-manual/toggle-manual'),
        html: require('!!raw-loader!./demos/toggle-manual/toggle-manual.html'),
        outlet: ToggleManualDemoComponent
      },
      {
        title: 'Inline display',
        anchor: 'inline-display',
        component: require('!!raw-loader!./demos/inline-display/inline-display'),
        html: require('!!raw-loader!./demos/inline-display/inline-display.html'),
        outlet: InlineDisplayDemoComponent
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
    usage: require('!!raw-loader!./docs/usage.md'),
    importInfo: '<span class="pln">ng add ngx</span><span class="pun">-</span><span class="pln">bootstrap </span> --component <span class="pln">collapse</span>',
    outlet: ApiSectionsComponent,
    content: [
      {
        title: 'CollapseDirective',
        anchor: 'collapse-directive',
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
        outlet: CollapseDemoComponent
      },
      {
        title: 'With animation',
        anchor: 'animated-ex',
        outlet: CollapseDemoAnimatedComponent
      },
      {
        title: 'Events',
        anchor: 'events-ex',
        outlet: CollapseDemoEventsComponent
      },
      {
        title: 'Manual toggle',
        anchor: 'manual-toggle-ex',
        outlet: ToggleManualDemoComponent
      },
      {
        title: 'Inline display',
        anchor: 'inline-display-ex',
        outlet: InlineDisplayDemoComponent
      }
    ]
  }
];
