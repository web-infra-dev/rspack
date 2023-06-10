import { DemoDropdownBasicComponent } from './demos/basic/basic';
import { DemoDropdownAnchorTriggerComponent } from './demos/anchor-trigger/anchor-trigger';
import { DemoDropdownSplitComponent } from './demos/split/split';
import { DemoDropdownTriggersManualComponent } from './demos/triggers-manual/triggers-manual';
import { DemoDropdownByIsOpenPropComponent } from './demos/trigger-by-isopen-property/trigger-by-isopen-property';
import { DemoDropdownDisabledComponent } from './demos/disabled-menu/disabled-menu';
import { DemoDropdownDisabledItemComponent } from './demos/disabled-item/disabled-item';
import { DemoDropdownAlignmentComponent } from './demos/alignment/menu-alignment';
import { DemoNestedDropdownsComponent } from './demos/nested-dropdowns/nested-dropdowns';
import { DemoDropdownContainerComponent } from './demos/container/container';
import { DemoDropdownDropupComponent } from './demos/dropup/dropup';
import { DemoDropdownMenuDividersComponent } from './demos/menu-dividers/menu-dividers';
import { DemoDropdownConfigComponent } from './demos/config/config';
import { DemoDropdownVisibilityEventsComponent } from './demos/visibility-events/visibility-events';
import { DemoDropdownStateChangeEventComponent } from './demos/state-change-event/state-change-event';
import { DemoDropdownAutoCloseComponent } from './demos/autoclose/autoclose';
import { DemoDropdownCustomHtmlComponent } from './demos/custom-html/custom-html';
import { DemoAccessibilityComponent } from './demos/accessibility/accessibility';
import { DemoDropdownInsideClickComponent } from './demos/inside-click/inside-click';

import { ContentSection } from '../../common-docs';
import { ExamplesComponent } from '../../common-docs';
import { ApiSectionsComponent } from '../../common-docs';

import {
  NgApiDocComponent,
  NgApiDocConfigComponent
} from '../../common-docs';

import { DemoDropdownAnimatedComponent } from './demos/animated/animated';


export const demoComponentContent: ContentSection[] = [
  {
    name: 'Overview',
    anchor: 'overview',
    tabName: 'overview',
    outlet: ExamplesComponent,
    description: `<p>Wrap the dropdown’s toggle (your button or link) and the dropdown menu within
      <code>dropdown</code>. Dropdowns can be triggered from <code> &lt;a&gt;</code> or <code> &lt;button&gt;</code>
      elements to better fit your potential needs.</p>`,
    content: [
      {
        title: 'Basic',
        anchor: 'basic',
        component: require('!!raw-loader!./demos/basic/basic.ts'),
        html: require('!!raw-loader!./demos/basic/basic.html'),
        description: `<p>Any <code>&lt;button&gt;</code> can became a dropdown toggle with few markup changes.
          Here’s how dropdown works with single button</p>`,
        outlet: DemoDropdownBasicComponent
      },
      {
        title: 'With animation',
        anchor: 'animated',
        component: require('!!raw-loader!./demos/animated/animated'),
        html: require('!!raw-loader!./demos/animated/animated.html'),
        description: `You can enable animation via <code>isAnimated</code> input or config option`,
        outlet: DemoDropdownAnimatedComponent
      },
      {
        title: 'Trigger by tag <a>',
        anchor: 'anchor-trigger',
        component: require('!!raw-loader!./demos/anchor-trigger/anchor-trigger.ts'),
        html: require('!!raw-loader!./demos/anchor-trigger/anchor-trigger.html'),
        outlet: DemoDropdownAnchorTriggerComponent
      },
      {
        title: 'Split button dropdowns',
        anchor: 'split-button',
        component: require('!!raw-loader!./demos/split/split.ts'),
        html: require('!!raw-loader!./demos/split/split.html'),
        description: `<p>Similarly, create split button dropdowns with virtually the same markup as single
          button dropdowns, but with the addition of <code>.dropdown-toggle-split</code> for proper spacing
          around the dropdown caret.</p>`,
        outlet: DemoDropdownSplitComponent
      },
      {
        title: 'Manual triggering',
        anchor: 'triggers-manual',
        component: require('!!raw-loader!./demos/triggers-manual/triggers-manual.ts'),
        html: require('!!raw-loader!./demos/triggers-manual/triggers-manual.html'),
        description: `<p>Dropdown can be triggered by <code>show</code>, <code>hide</code> and
          <code>toggle</code> methods from directive
          <br>
          Use method <code>toggle(true)</code> if you want to toggle the dropdown or <code>toggle(false)</code>
          if you want to only close opened dropdown.
          </p>`,
        outlet: DemoDropdownTriggersManualComponent
      },
      {
        title: 'Trigger by isOpen property',
        anchor: 'trigger-by-isopen-property',
        component: require('!!raw-loader!./demos/trigger-by-isopen-property/trigger-by-isopen-property.ts'),
        html: require('!!raw-loader!./demos/trigger-by-isopen-property/trigger-by-isopen-property.html'),
        description: `<p>Dropdown can be shown or hidden by changing <code>isOpen</code> input property</p>`,
        outlet: DemoDropdownByIsOpenPropComponent
      },
      {
        title: 'Disabled menu',
        anchor: 'disabled-menu',
        component: require('!!raw-loader!./demos/disabled-menu/disabled-menu.ts'),
        html: require('!!raw-loader!./demos/disabled-menu/disabled-menu.html'),
        description: `<p>Use <code>isDisabled</code> property to make dropdown disabled.</p>`,
        outlet: DemoDropdownDisabledComponent
      },
      {
        title: 'Mark item as disabled',
        anchor: 'disabled-item',
        component: require('!!raw-loader!./demos/disabled-item/disabled-item.ts'),
        html: require('!!raw-loader!./demos/disabled-item/disabled-item.html'),
        description: `<p>Add a <code>disabled</code> class to <code>&lt;a&gt;</code> to make it as disabled.</p>`,
        outlet: DemoDropdownDisabledItemComponent
      },
      {
        title: 'Menu alignment',
        anchor: 'menu-alignment',
        component: require('!!raw-loader!./demos/alignment/menu-alignment.ts'),
        html: require('!!raw-loader!./demos/alignment/menu-alignment.html'),
        description: `<p>By default, a dropdown menu is automatically positioned 100% from the top and along
          the left side of its parent. Add class <code>.dropdown-menu-right</code> to a <code>dropdownMenu</code>
          to right align the dropdown menu.</p>`,
        outlet: DemoDropdownAlignmentComponent
      },
      {
        title: 'Inside click',
        anchor: 'inside-click',
        component: require('!!raw-loader!./demos/inside-click/inside-click.ts'),
        html: require('!!raw-loader!./demos/inside-click/inside-click.html'),
        description: `<p>By default, a dropdown menu closes on document click, even if you clicked on an element inside the dropdown.
        Use <code>[insideClick]="true"</code> to allow click inside the dropdown</p>`,
        outlet: DemoDropdownInsideClickComponent
      },
      {
        title: 'Nested dropdowns (experimental)',
        anchor: 'nested-dropdowns',
        component: require('!!raw-loader!./demos/nested-dropdowns/nested-dropdowns.ts'),
        html: require('!!raw-loader!./demos/nested-dropdowns/nested-dropdowns.html'),
        outlet: DemoNestedDropdownsComponent
      },
      {
        title: 'Append to body',
        anchor: 'container',
        component: require('!!raw-loader!./demos/container/container.ts'),
        html: require('!!raw-loader!./demos/container/container.html'),
        description: `<p>Append dropdown to body by adding <code>container="body"</code> to the parent element.</p>`,
        outlet: DemoDropdownContainerComponent
      },
      /* not availavle in bs-dropdown version
      {
        title: 'Single button with keyboard nav',
        anchor: 'dropdown-keyboard',
        component: require('!!raw-loader!./demos/keyboard/keyboard.ts'),
        html: require('!!raw-loader!./demos/keyboard/keyboard.html'),
        outlet: DemoDropdownKeyboardComponent
      },*/
      {
        title: 'Dropup variation',
        anchor: 'dropup',
        component: require('!!raw-loader!./demos/dropup/dropup.ts'),
        html: require('!!raw-loader!./demos/dropup/dropup.html'),
        description: `<p>To make dropdown's menu appear above toggle element set <code>dropup</code> property as <code>true</code></p>`,
        outlet: DemoDropdownDropupComponent
      },
      {
        title: 'Menu dividers',
        anchor: 'menu-dividers',
        component: require('!!raw-loader!./demos/menu-dividers/menu-dividers.ts'),
        html: require('!!raw-loader!./demos/menu-dividers/menu-dividers.html'),
        description: `<p>Separate groups of related menu items with a <code>.dropdown-divider</code> for bootstrap 4.</p>`,
        outlet: DemoDropdownMenuDividersComponent
      },
      {
        title: 'Custom html',
        anchor: 'custom-html',
        component: require('!!raw-loader!./demos/custom-html/custom-html.ts'),
        html: require('!!raw-loader!./demos/custom-html/custom-html.html'),
        description: `<p>Dropdown allows you to use any html markup inside of it</p>`,
        outlet: DemoDropdownCustomHtmlComponent
      },
      {
        title: 'Configuring defaults',
        anchor: 'config-defaults',
        component: require('!!raw-loader!./demos/config/config.ts'),
        html: require('!!raw-loader!./demos/config/config.html'),
        description: `<p>It is possible to override default dropdown config partially or completely.</p>`,
        outlet: DemoDropdownConfigComponent
      },
      {
        title: 'Visibility Events',
        anchor: 'visibility-events',
        component: require('!!raw-loader!./demos/visibility-events/visibility-events.ts'),
        html: require('!!raw-loader!./demos/visibility-events/visibility-events.html'),
        description: `<p>You can subscribe to dropdown's visibility events</p>`,
        outlet: DemoDropdownVisibilityEventsComponent
      },
      {
        title: 'State change event',
        anchor: 'state-change-event',
        component: require('!!raw-loader!./demos/state-change-event/state-change-event.ts'),
        html: require('!!raw-loader!./demos/state-change-event/state-change-event.html'),
        description: `<p>You can subscribe to dropdown's state change event (<code>isOpenChange</code>).</p>`,
        outlet: DemoDropdownStateChangeEventComponent
      },
      {
        title: 'Auto close',
        anchor: 'autoclose',
        component: require('!!raw-loader!./demos/autoclose/autoclose.ts'),
        html: require('!!raw-loader!./demos/autoclose/autoclose.html'),
        description: `<p>Use <code>autoClose</code> property to change dropdown's default behavior</p>`,
        outlet: DemoDropdownAutoCloseComponent
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
    importInfo: '<span class="pln">ng add ngx</span><span class="pun">-</span><span class="pln">bootstrap </span> --component <span class="pln">dropdowns</span>',
    content: [
      {
        title: 'BsDropdownDirective',
        anchor: 'dropdown-directive',
        outlet: NgApiDocComponent
      },
      {
        title: 'BsDropdownContainerComponent',
        anchor: 'dropdown-container',
        outlet: NgApiDocComponent
      },
      {
        title: 'BsDropdownMenuDirective',
        anchor: 'dropdown-menu-directive',
        outlet: NgApiDocComponent
      },
      {
        title: 'BsDropdownToggleDirective',
        anchor: 'dropdown-toggle-directive',
        outlet: NgApiDocComponent
      },
      {
        title: 'BsDropdownState',
        anchor: 'BsDropdownState',
        outlet: NgApiDocConfigComponent
      },
      {
        title: 'BsDropdownConfig',
        anchor: 'dropdown-config',
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
        outlet: DemoDropdownBasicComponent
      },
      {
        title: 'With animation',
        anchor: 'animated-ex',
        outlet: DemoDropdownAnimatedComponent
      },
      {
        title: 'Trigger by tag <a>',
        anchor: 'anchor-trigger-ex',
        outlet: DemoDropdownAnchorTriggerComponent
      },
      {
        title: 'Split button dropdowns',
        anchor: 'split-button-ex',
        outlet: DemoDropdownSplitComponent
      },
      {
        title: 'Manual triggering',
        anchor: 'triggers-manual-ex',
        outlet: DemoDropdownTriggersManualComponent
      },
      {
        title: 'Trigger by isOpen property',
        anchor: 'trigger-by-isopen-property-ex',
        outlet: DemoDropdownByIsOpenPropComponent
      },
      {
        title: 'Disabled menu',
        anchor: 'disabled-menu-ex',
        outlet: DemoDropdownDisabledComponent
      },
      {
        title: 'Mark item as disabled',
        anchor: 'disabled-item-ex',
        outlet: DemoDropdownDisabledItemComponent
      },
      {
        title: 'Menu alignment',
        anchor: 'menu-alignment-ex',
        outlet: DemoDropdownAlignmentComponent
      },
      {
        title: 'Inside click',
        anchor: 'inside-click-ex',
        outlet: DemoDropdownInsideClickComponent
      },
      {
        title: 'Nested dropdowns (experimental)',
        anchor: 'nested-dropdowns-ex',
        outlet: DemoNestedDropdownsComponent
      },
      {
        title: 'Append to body',
        anchor: 'container-ex',
        outlet: DemoDropdownContainerComponent
      },
      {
        title: 'Dropup variation',
        anchor: 'dropup-ex',
        outlet: DemoDropdownDropupComponent
      },
      {
        title: 'Menu dividers',
        anchor: 'menu-dividers-ex',
        outlet: DemoDropdownMenuDividersComponent
      },
      {
        title: 'Custom html',
        anchor: 'custom-html-ex',
        outlet: DemoDropdownCustomHtmlComponent
      },
      {
        title: 'Configuring defaults',
        anchor: 'config-defaults-ex',
        outlet: DemoDropdownConfigComponent
      },
      {
        title: 'Visibility Events',
        anchor: 'visibility-events-ex',
        outlet: DemoDropdownVisibilityEventsComponent
      },
      {
        title: 'State change event',
        anchor: 'state-change-event-ex',
        outlet: DemoDropdownStateChangeEventComponent
      },
      {
        title: 'Auto close',
        anchor: 'autoclose-ex',
        outlet: DemoDropdownAutoCloseComponent
      }
    ]
  }
];
