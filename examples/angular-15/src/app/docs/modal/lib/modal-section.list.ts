import { DemoModalServiceStaticComponent } from './demos/service-template/service-template';
import { DemoModalServiceFromComponent } from './demos/service-component/service-component';
import { DemoModalServiceNestedComponent } from './demos/service-nested/service-nested';
import { DemoModalServiceEventsComponent } from './demos/service-events/service-events';
import { DemoModalServiceDisableAnimationComponent } from './demos/service-options/disable-animation/disable-animation';
import { DemoModalServiceCustomCSSClassComponent } from './demos/service-options/custom-css-class/custom-css-class';
import { DemoModalServiceDisableEscClosingComponent } from './demos/service-options/disable-esc-closing/disable-esc-closing';
import { DemoModalServiceDisableBackdropComponent } from './demos/service-options/disable-backdrop/disable-backdrop';
import { DemoModalServiceConfirmWindowComponent } from './demos/service-confirm-window/service-confirm-window';
import { DemoModalServiceChangeClassComponent } from './demos/service-options/change-class/change-class';

import { DemoModalStaticComponent } from './demos/static/static';
import { DemoModalSizesComponent } from './demos/sizes/sizes';
import { DemoModalChildComponent } from './demos/child/child';
import { DemoModalNestedComponent } from './demos/nested/nested';
import { DemoModalEventsComponent } from './demos/events/events';
import { DemoAutoShownModalComponent } from './demos/auto-shown/auto-shown';
import { DemoAccessibilityComponent } from './demos/accessibility/accessibility';
import { DemoModalWithPopupsComponent } from './demos/modal-with-popups/modal-with-popups';

import { ContentSection } from '../../common-docs';
import { ExamplesComponent } from '../../common-docs';
import { ApiSectionsComponent } from '../../common-docs';

import {
  NgApiDocComponent,
  NgApiDocClassComponent,
  NgApiDocConfigComponent
} from '../../common-docs';
import { DemoModalScrollingLongContentComponent } from './demos/scrolling-long-content/scrolling-long-content';
import { DemoModalRefEventsComponent } from './demos/modal-ref-events/modal-ref-events';
import { DemoModalServiceWithInterceptorComponent } from './demos/service-interceptor/service-interceptor';

export const demoComponentContent: ContentSection[] = [
  {
    name: 'Overview',
    anchor: 'overview',
    tabName: 'overview',
    outlet: ExamplesComponent,
    content: [
      {
        title: 'Service examples',
        anchor: 'service-section',
        description: `<p>Open a modal from service.</p>
      <p>To be able to open modals from service, inject <code>BsModalService</code> to your constructor.<br>Then, call
      <code>.show()</code> method of modal service. Pass a <code>TemplateRef</code> or a component as a first argument and
      config as a second (optionally). <br> <code>.show()</code> method returns an instance of <code>BsModalRef</code>
      class with <code>.hide()</code> method and <code>content</code> property where you'll find a component
      which you've passed to service.</p>`
      },
      {
        title: 'Template',
        anchor: 'service-template',
        component: require('!!raw-loader!./demos/service-template/service-template.ts'),
        html: require('!!raw-loader!./demos/service-template/service-template.html'),
        outlet: DemoModalServiceStaticComponent
      },
      {
        title: 'Component',
        anchor: 'service-component',
        component: require('!!raw-loader!./demos/service-component/service-component.ts'),
        html: require('!!raw-loader!./demos/service-component/service-component.html'),
        description: `<p>Creating a modal with component just as easy as it is with template. Just pass your component
          in <code>.show()</code> method as in example, and don't forget to include your component to
          <code>entryComponents</code> of your <code>NgModule</code><br> If you passed a component
          to <code>.show()</code> you can get access to opened modal by injecting <code>BsModalRef</code>. Also you can pass data
          in your modal by adding <code>initialState</code> field in config. See example for more info</p>`,
        outlet: DemoModalServiceFromComponent
      },
      {
        title: 'Nested',
        anchor: 'service-nested',
        component: require('!!raw-loader!./demos/service-nested/service-nested.ts'),
        html: require('!!raw-loader!./demos/service-nested/service-nested.html'),
        description: `<p>Nested modals are supported</p>`,
        outlet: DemoModalServiceNestedComponent
      },
      {
        title: 'Scrolling long content',
        anchor: 'scrolling-long-content',
        component: require('!!raw-loader!./demos/scrolling-long-content/scrolling-long-content.ts'),
        html: require('!!raw-loader!./demos/scrolling-long-content/scrolling-long-content.html'),
        outlet: DemoModalScrollingLongContentComponent
      },
      {
        title: 'Events',
        anchor: 'service-events',
        component: require('!!raw-loader!./demos/service-events/service-events.ts'),
        html: require('!!raw-loader!./demos/service-events/service-events.html'),
        description: `
          <p>Modal service events. Modal service exposes 4 events: <code>onShow</code>, <code>onShown</code>,
          <code>onHide</code>, <code>onHidden</code>.
          See usage example below.</p>
          <p><code>onHide</code> and <code>onHidden</code> events emit dismiss reason. Possible values are
          <code>backdrop-click</code>, <code>esc</code> or <code>{id: number | string}</code> if modal was closed by direct call of
          <code>hide()</code> method</p>`,
        outlet: DemoModalServiceEventsComponent
      },
      {
        title: 'ModalRef Events',
        anchor: 'modal-ref-events',
        component: require('!!raw-loader!./demos/modal-ref-events/modal-ref-events.ts'),
        html: require('!!raw-loader!./demos/modal-ref-events/modal-ref-events.html'),
        description: `
          <p>Modal ref events. ModalRef exposes 2 events: <code>onHide</code> and <code>onHidden</code>. Note,
          <code>onShow</code> and <code>onShown</code> are not options because they have already fired by the time
          the ModalRef is created.
          See usage example below.</p>
          <p><code>onHide</code> and <code>onHidden</code> events emit dismiss reason. Possible values are
          <code>backdrop-click</code>, <code>esc</code> or <code>{id: number | string}</code> if modal was closed by direct call of
          <code>hide()</code> method</p>`,
        outlet: DemoModalRefEventsComponent
      },
      {
        title: 'Confirm Window',
        anchor: 'confirm-window',
        component: require('!!raw-loader!./demos/service-confirm-window/service-confirm-window.ts'),
        html: require('!!raw-loader!./demos/service-confirm-window/service-confirm-window.html'),
        description: `<p>Modal with opportunity to <code>confirm</code> or <code>decline</code>.</p>`,
        outlet: DemoModalServiceConfirmWindowComponent
      },
      {
        title: 'Сustom css class',
        anchor: 'service-custom-css-class',
        component: require('!!raw-loader!./demos/service-options/custom-css-class/custom-css-class.ts'),
        html: require('!!raw-loader!./demos/service-options/custom-css-class/custom-css-class.html'),
        description: `<p>There is possibility to add custom css class to a modal.
          See the demo below to learn how to use it</p>`,
        outlet: DemoModalServiceCustomCSSClassComponent
      },
      {
        title: 'Animation option',
        anchor: 'service-disable-animation',
        component: require('!!raw-loader!./demos/service-options/disable-animation/disable-animation.ts'),
        html: require('!!raw-loader!./demos/service-options/disable-animation/disable-animation.html'),
        description: `<p>There is animation option that you can configure.</p>`,
        outlet: DemoModalServiceDisableAnimationComponent
      },
      {
        title: 'Esc closing option',
        anchor: 'service-disable-esc-closing',
        component: require('!!raw-loader!./demos/service-options/disable-esc-closing/disable-esc-closing.ts'),
        html: require('!!raw-loader!./demos/service-options/disable-esc-closing/disable-esc-closing.html'),
        description: `<p>There is closing by Esc button option that you can configure.</p>`,
        outlet: DemoModalServiceDisableEscClosingComponent
      },
      {
        title: 'Modal window with tooltip and popover',
        anchor: 'modal-with-popups',
        component: require('!!raw-loader!./demos/modal-with-popups/modal-with-popups.ts'),
        html: require('!!raw-loader!./demos/modal-with-popups/modal-with-popups.html'),
        description: `<p><code>Tooltips</code> and <code>popovers</code> can be placed within modals as needed. When modals are closed, any <code>tooltips</code> and <code>popovers</code> within are also automatically dismissed.</p>`,
        outlet: DemoModalWithPopupsComponent
      },
      {
        title: 'Backdrop options',
        anchor: 'service-disable-backdrop',
        component: require('!!raw-loader!./demos/service-options/disable-backdrop/disable-backdrop.ts'),
        html: require('!!raw-loader!./demos/service-options/disable-backdrop/disable-backdrop.html'),
        description: `<p>There is backdrop options that you can configure.</p>`,
        outlet: DemoModalServiceDisableBackdropComponent
      },
      {
        title: 'Change class',
        anchor: 'change-class',
        component: require('!!raw-loader!./demos/service-options/change-class/change-class.ts'),
        html: require('!!raw-loader!./demos/service-options/change-class/change-class.html'),
        description: `<p>Calling setClass method to change modal's window class</p>`,
        outlet: DemoModalServiceChangeClassComponent
      },
      {
        title: 'Close interceptor',
        anchor: 'service-with-interceptor',
        component: require('!!raw-loader!./demos/service-interceptor/service-interceptor.ts'),
        html: require('!!raw-loader!./demos/service-interceptor/service-interceptor.html'),
        description: `<p>When opening a modal with a component, you can provide an interceptor which will be triggered
          whenever the modal try to close, allowing you to block the disappearance of a modal.</p>`,
        outlet: DemoModalServiceWithInterceptorComponent
      },
      {
        title: 'Directive examples',
        anchor: 'directive-section',
        description: `<p>Also you can use directive instead of service. See the demos below </p>`
      },
      {
        title: 'Static modal',
        anchor: 'directive-static',
        component: require('!!raw-loader!./demos/static/static.ts'),
        html: require('!!raw-loader!./demos/static/static.html'),
        outlet: DemoModalStaticComponent
      },
      {
        title: 'Optional sizes',
        anchor: 'directive-sizes',
        component: require('!!raw-loader!./demos/sizes/sizes.ts'),
        html: require('!!raw-loader!./demos/sizes/sizes.html'),
        outlet: DemoModalSizesComponent
      },
      {
        title: 'Child modal',
        anchor: 'directive-child',
        component: require('!!raw-loader!./demos/child/child.ts'),
        html: require('!!raw-loader!./demos/child/child.html'),
        description: `<p>Control modal from parent component</p>`,
        outlet: DemoModalChildComponent
      },
      {
        title: 'Nested modals',
        anchor: 'directive-nested',
        component: require('!!raw-loader!./demos/nested/nested.ts'),
        html: require('!!raw-loader!./demos/nested/nested.html'),
        description: `<p>Open a modal from another modal</p>`,
        outlet: DemoModalNestedComponent
      },
      {
        title: 'Modal events',
        anchor: 'directive-events',
        component: require('!!raw-loader!./demos/events/events.ts'),
        html: require('!!raw-loader!./demos/events/events.html'),
        description: `<p><code>ModalDirective</code> exposes 4 events: <code>onShow</code>, <code>onShown</code>,
          <code>onHide</code>, <code>onHidden</code>. See usage example below.<br>
          <code>$event</code> is an instance of <code>ModalDirective</code>. There you may
          find some useful properties like <code>isShown</code>, <code>dismissReason</code>, etc.
          <br>For example, you may want to know which one of user's actions caused closing of a modal.
          Just get the value of <code>dismissReason</code>,<br> possible values are <code>backdrop-click</code>,
          <code>esc</code> or <code>null</code> if modal was closed by direct call of <code>hide()</code> method</p>`,
        outlet: DemoModalEventsComponent
      },
      {
        title: 'Auto shown modal',
        anchor: 'directive-auto-shown',
        component: require('!!raw-loader!./demos/auto-shown/auto-shown.ts'),
        html: require('!!raw-loader!./demos/auto-shown/auto-shown.html'),
        description: `
          <p>Show modal right after it has been initialized. This allows you to keep DOM clean by only
          appending visible modals to the DOM using <code>*ngIf</code> directive.</p>
          <p>It can also be useful if you want your modal component to perform some initialization operations, but
          want to defer that until user actually sees modal content. I.e. for a "Select e-mail recipient" modal
          you might want to defer recipient list loading until the modal is shown.</p>`,
        outlet: DemoAutoShownModalComponent
      },
      {
        title: 'Accessibility',
        anchor: 'accessibility',
        component: require('!!raw-loader!./demos/accessibility/accessibility.ts'),
        html: require('!!raw-loader!./demos/accessibility/accessibility.html'),
        description: `
        <p>
          Be sure to add <code class="highlighter-rouge">id=""</code> attribute to your title and description
          in the template to make your modal works according to accessibility. The <code class="highlighter-rouge">aria-labelledby</code>
          attribute establishes relationships between the modal and its title (only if the title has id attribute). The element
          containing the modal's description is referenced by <code class="highlighter-rouge">aria-describedby</code> attribute.
          The dialog does not need <code class="highlighter-rouge">aria-describedby</code> since there is no static
          text that describes it.
        </p>
        <p>
        Use modal options to set <code class="highlighter-rouge">aria-labelledby</code> and
        <code class="highlighter-rouge">aria-describedby</code> attributes.
        </p>
        `,
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
    importInfo: '<span class="pln">ng add ngx</span><span class="pun">-</span><span class="pln">bootstrap </span> --component <span class="pln">modals</span>',
    content: [
      {
        title: 'ModalDirective',
        anchor: 'modal-directive',
        outlet: NgApiDocComponent
      },
      {
        title: 'ModalBackdropComponent',
        anchor: 'modal-backdrop-component',
        outlet: NgApiDocComponent
      },
      {
        title: 'BsModalService',
        anchor: 'bs-modal-service',
        outlet: NgApiDocClassComponent
      },
      {
        title: 'BsModalRef',
        anchor: 'bs-modal-ref',
        outlet: NgApiDocClassComponent
      },
      {
        title: 'ModalOptions',
        anchor: 'modal-options',
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
        title: 'Template',
        anchor: 'service-template-ex',
        outlet: DemoModalServiceStaticComponent
      },
      {
        title: 'Component',
        anchor: 'service-component-ex',
        outlet: DemoModalServiceFromComponent
      },
      {
        title: 'Nested',
        anchor: 'service-nested-ex',
        outlet: DemoModalServiceNestedComponent
      },
      {
        title: 'Scrolling long content',
        anchor: 'scrolling-long-content-ex',
        outlet: DemoModalScrollingLongContentComponent
      },
      {
        title: 'Events',
        anchor: 'service-events-ex',
        outlet: DemoModalServiceEventsComponent
      },
      {
        title: 'ModalRef Events',
        anchor: 'modal-ref-events-ex',
        outlet: DemoModalRefEventsComponent
      },
      {
        title: 'Confirm Window',
        anchor: 'confirm-window-ex',
        outlet: DemoModalServiceConfirmWindowComponent
      },
      {
        title: 'Сustom css class',
        anchor: 'service-custom-css-class-ex',
        outlet: DemoModalServiceCustomCSSClassComponent
      },
      {
        title: 'Animation option',
        anchor: 'service-disable-animation-ex',
        outlet: DemoModalServiceDisableAnimationComponent
      },
      {
        title: 'Esc closing option',
        anchor: 'service-disable-esc-closing-ex',
        outlet: DemoModalServiceDisableEscClosingComponent
      },
      {
        title: 'Modal window with tooltip and popover',
        anchor: 'modal-with-popups-ex',
        outlet: DemoModalWithPopupsComponent
      },
      {
        title: 'Backdrop options',
        anchor: 'service-disable-backdrop-ex',
        outlet: DemoModalServiceDisableBackdropComponent
      },
      {
        title: 'Change class',
        anchor: 'change-class-ex',
        outlet: DemoModalServiceChangeClassComponent
      },
      {
        title: 'Close interceptor',
        anchor: 'service-with-interceptor-ex',
        outlet: DemoModalServiceWithInterceptorComponent
      },
      {
        title: 'Static modal',
        anchor: 'directive-static-ex',
        outlet: DemoModalStaticComponent
      },
      {
        title: 'Optional sizes',
        anchor: 'directive-sizes-ex',
        outlet: DemoModalSizesComponent
      },
      {
        title: 'Child modal',
        anchor: 'directive-child-ex',
        outlet: DemoModalChildComponent
      },
      {
        title: 'Nested modals',
        anchor: 'directive-nested-ex',
        outlet: DemoModalNestedComponent
      },
      {
        title: 'Modal events',
        anchor: 'directive-events-ex',
        outlet: DemoModalEventsComponent
      },
      {
        title: 'Auto shown modal',
        anchor: 'directive-auto-shown-ex',
        outlet: DemoAutoShownModalComponent
      }
    ]
  }
];
