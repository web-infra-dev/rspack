import { ApiSectionsComponent } from '../../common-docs';
import { ContentSection } from '../../common-docs';
import { DemoTypeaheadAdaptivePositionComponent } from './demos/adaptive-position/adaptive-position';
import { DemoTypeaheadAnimatedComponent } from './demos/animated/animated';
import { DemoTypeaheadBasicComponent } from './demos/basic/basic';
import { DemoTypeaheadCancelRequestOnFocusLostComponent } from './demos/cancel-on-focus-lost/cancel-on-focus-lost';
import { DemoTypeaheadConfigComponent } from './demos/config/config';
import { DemoTypeaheadContainerComponent } from './demos/container/container';
import { DemoTypeaheadDelayComponent } from './demos/delay/delay';
import { DemoTypeaheadDropupComponent } from './demos/dropup/dropup';
import { DemoTypeaheadFieldComponent } from './demos/field/field';
import { DemoTypeaheadAsyncComponent } from './demos/async/async';
import { DemoTypeaheadReactiveFormComponent } from './demos/reactive-form/reactive-form';
import { DemoTypeaheadFormComponent } from './demos/form/form';
import { DemoTypeaheadGroupingComponent } from './demos/grouping/grouping';
import { DemoTypeaheadItemTemplateComponent } from './demos/item-template/item-template';
import { DemoTypeaheadListTemplateComponent } from './demos/list-template/list-template';
import { DemoTypeaheadLatinizeComponent } from './demos/latinize/latinize';
import { DemoTypeaheadMinLengthComponent } from './demos/min-length/min-length';
import { DemoTypeaheadNoResultComponent } from './demos/no-result/no-result';
import { DemoTypeaheadOnBlurComponent } from './demos/on-blur/on-blur';
import { DemoTypeaheadOnSelectComponent } from './demos/on-select/on-select';
import { DemoTypeaheadPhraseDelimitersComponent } from './demos/phrase-delimiters/phrase-delimiters';
import { DemoTypeaheadScrollableComponent } from './demos/scrollable/scrollable';
import { DemotypeaheadSelectFirstItemComponent } from './demos/selected-first-item/selected-first-item';
import { DemoTypeaheadShowOnBlurComponent } from './demos/show-on-blur/show-on-blur';
import { DemoTypeaheadSingleWorldComponent } from './demos/single-world/single-world';
import { ExamplesComponent } from '../../common-docs';

import { NgApiDocComponent, NgApiDocConfigComponent } from '../../common-docs';
import { DemoTypeaheadFirstItemActiveComponent } from './demos/first-item-active/first-item-active';
import { DemoTypeaheadAsyncHttpRequestComponent } from './demos/async-http-request/async-http-request';
import { DemoTypeaheadOrderingComponent } from './demos/ordering/ordering';
import { DemoTypeaheadMultipleSearchComponent } from './demos/multiple-search/multiple-search';

export const demoComponentContent: ContentSection[] = [
  {
    name: 'Overview',
    anchor: 'overview',
    tabName: 'overview',
    outlet: ExamplesComponent,
    content: [
      {
        title: 'Basic array',
        anchor: 'Basic-array',
        component: require('!!raw-loader!./demos/basic/basic.ts'),
        html: require('!!raw-loader!./demos/basic/basic.html'),
        outlet: DemoTypeaheadBasicComponent
      },
      {
        title: 'With animation',
        anchor: 'animated',
        component: require('!!raw-loader!./demos/animated/animated'),
        html: require('!!raw-loader!./demos/animated/animated.html'),
        description: `You can enable animation via <code>isAnimated</code> input or config option`,
        outlet: DemoTypeaheadAnimatedComponent
      },
      {
        title: 'Adaptive position',
        anchor: 'adaptive-position',
        description: `
          <p>You can enable adaptive position via <code>adaptivePosition</code> input or config option</p>
        `,
        component: require('!!raw-loader!./demos/adaptive-position/adaptive-position.ts'),
        html: require('!!raw-loader!./demos/adaptive-position/adaptive-position.html'),
        outlet: DemoTypeaheadAdaptivePositionComponent
      },
      {
        title: 'Item template',
        anchor: 'item-template',
        component: require('!!raw-loader!./demos/item-template/item-template.ts'),
        html: require('!!raw-loader!./demos/item-template/item-template.html'),
        outlet: DemoTypeaheadItemTemplateComponent
      },
      {
        title: 'List template',
        anchor: 'list-template',
        component: require('!!raw-loader!./demos/list-template/list-template.ts'),
        html: require('!!raw-loader!./demos/list-template/list-template.html'),
        outlet: DemoTypeaheadListTemplateComponent
      },
      {
        title: 'Option field',
        anchor: 'option-field',
        component: require('!!raw-loader!./demos/field/field.ts'),
        html: require('!!raw-loader!./demos/field/field.html'),
        outlet: DemoTypeaheadFieldComponent
      },
      {
        title: 'Async data',
        anchor: 'async-data',
        component: require('!!raw-loader!./demos/async/async.ts'),
        html: require('!!raw-loader!./demos/async/async.html'),
        outlet: DemoTypeaheadAsyncComponent
      },
      {
        title: 'Async using http request',
        anchor: 'async-http-request',
        description: `
          <p>Use http request to search for data. If you need to handle http error, do this inside <code>tap</code> operator.
          Enter search value several times (10-15), and after a few success responses API should return an error
          (GitHub limit for requests)</p>
        `,
        component: require('!!raw-loader!./demos/async-http-request/async-http-request.ts'),
        html: require('!!raw-loader!./demos/async-http-request/async-http-request.html'),
        outlet: DemoTypeaheadAsyncHttpRequestComponent
      },
      {
        title: 'Cancel on focus lost',
        anchor: 'cancel-on-focus-lost',
        description: `<p>Set config property <code>cancelRequestOnFocusLost</code> to <code>true</code> if you want to cancel async request on focus lost event</p>`,
        component: require('!!raw-loader!./demos/cancel-on-focus-lost/cancel-on-focus-lost.ts'),
        html: require('!!raw-loader!./demos/cancel-on-focus-lost/cancel-on-focus-lost.html'),
        outlet: DemoTypeaheadCancelRequestOnFocusLostComponent
      },
      {
        title: 'With delay',
        anchor: 'delay',
        description: `
          <p>Use <code>typeaheadWaitMs</code> to set minimal waiting time after last character typed
          before typeahead kicks-in. In example a search begins with delay in 1 second</p>
        `,
        component: require('!!raw-loader!./demos/delay/delay.ts'),
        html: require('!!raw-loader!./demos/delay/delay.html'),
        outlet: DemoTypeaheadDelayComponent
      },
      {
        title: 'Template-driven forms',
        anchor: 'forms',
        description: `
          <p>Typeahead can be used in template-driven forms. Keep in mind that value of <code>ngModel</code> is
          <code>string</code></p>
        `,
        component: require('!!raw-loader!./demos/form/form.ts'),
        html: require('!!raw-loader!./demos/form/form.html'),
        outlet: DemoTypeaheadFormComponent
      },
      {
        title: 'Reactive forms',
        anchor: 'reactive-forms',
        description: `
          <p>Typeahead can be used in reactive forms</p>
        `,
        component: require('!!raw-loader!./demos/reactive-form/reactive-form.ts'),
        html: require('!!raw-loader!./demos/reactive-form/reactive-form.html'),
        outlet: DemoTypeaheadReactiveFormComponent
      },
      {
        title: 'Grouping results',
        anchor: 'grouping-results',
        component: require('!!raw-loader!./demos/grouping/grouping.ts'),
        html: require('!!raw-loader!./demos/grouping/grouping.html'),
        outlet: DemoTypeaheadGroupingComponent
      },
      {
        title: 'Ignore spaces and order',
        anchor: 'single-world',
        component: require('!!raw-loader!./demos/single-world/single-world.ts'),
        html: require('!!raw-loader!./demos/single-world/single-world.html'),
        description: `
          <p>After setting <code>typeaheadSingleWords</code> input property to <code>true</code>
          order of typed symbols and spaces between them will be ignored. For example, "<i>zona ari</i>"
          will match with "<i>Arizona</i>"</p>
        `,
        outlet: DemoTypeaheadSingleWorldComponent
      },
      {
        title: 'Phrase delimiters',
        anchor: 'phrase-delimiters',
        component: require('!!raw-loader!./demos/phrase-delimiters/phrase-delimiters.ts'),
        html: require('!!raw-loader!./demos/phrase-delimiters/phrase-delimiters.html'),
        description: `
          <p>Set the word delimiter by <code>typeaheadPhraseDelimiters</code> to match exact phrase.
          This is demo with delimeters "<code>&</code>" and "<code>,</code>"</p>
        `,
        outlet: DemoTypeaheadPhraseDelimitersComponent
      },
      {
        title: 'Dropup',
        anchor: 'dropup',
        component: require('!!raw-loader!./demos/dropup/dropup.ts'),
        html: require('!!raw-loader!./demos/dropup/dropup.html'),
        outlet: DemoTypeaheadDropupComponent
      },
      {
        title: 'On blur',
        anchor: 'on-blur',
        description: `
         <p>Returns an option on which user lost a focus. To reproduce start typing the name of the state, then focus
         on one of the options with mouse or arrow keys and click outside of the typeahead</p>
        `,
        component: require('!!raw-loader!./demos/on-blur/on-blur.ts'),
        html: require('!!raw-loader!./demos/on-blur/on-blur.html'),
        outlet: DemoTypeaheadOnBlurComponent
      },
      {
        title: 'Append to body',
        anchor: 'container',
        description: `
        <p><code>container</code> is an input property specifying the element the typeahead should be appended to.</p>
        `,
        component: require('!!raw-loader!./demos/container/container.ts'),
        html: require('!!raw-loader!./demos/container/container.html'),
        outlet: DemoTypeaheadContainerComponent
      },
      {
        title: 'No result',
        anchor: 'no-result',
        description: `
         <p>Used to display the state when no matches were found. To see message
         "No Results Found" enter the value that doesn't match anything from the list</p>
        `,
        component: require('!!raw-loader!./demos/no-result/no-result.ts'),
        html: require('!!raw-loader!./demos/no-result/no-result.html'),
        outlet: DemoTypeaheadNoResultComponent
      },
      {
        title: 'Scrollable',
        anchor: 'scrollable',
        component: require('!!raw-loader!./demos/scrollable/scrollable.ts'),
        html: require('!!raw-loader!./demos/scrollable/scrollable.html'),
        outlet: DemoTypeaheadScrollableComponent
      },
      {
        title: 'Latinize',
        anchor: 'latinize',
        description: `
          <p>Use <code>typeaheadLatinize</code> property for matching latin symbols. If it is set
          to <code>true</code> the word <strong>súper</strong> would match <strong>super</strong> and vice versa.</p>
        `,
        component: require('!!raw-loader!./demos/latinize/latinize.ts'),
        html: require('!!raw-loader!./demos/latinize/latinize.html'),
        outlet: DemoTypeaheadLatinizeComponent
      },
      {
        title: 'On select / On preview',
        anchor: 'on-select',
        description: `
          <p><code>typeaheadOnSelect</code> event is fired when an option was selected.
          Returns an object with this option.</p>
          <p><code>typeaheadOnPreview</code> event is fired when an option was highlighted.
          Returns an object with this option.</p>
        `,
        component: require('!!raw-loader!./demos/on-select/on-select.ts'),
        html: require('!!raw-loader!./demos/on-select/on-select.html'),
        outlet: DemoTypeaheadOnSelectComponent
      },
      {
        title: 'Min length',
        anchor: 'min-length',
        description: `
          <p>Minimal number of characters that needs to be entered before typeahead kicks in. When set to 0, typeahead shows on focus with full list of options.</p>
        `,
        component: require('!!raw-loader!./demos/min-length/min-length.ts'),
        html: require('!!raw-loader!./demos/min-length/min-length.html'),
        outlet: DemoTypeaheadMinLengthComponent
      },
      {
        title: 'Show results on blur',
        anchor: 'show-on-blur',
        description: `
          <p>Use input property <code>typeaheadHideResultsOnBlur</code> or config property <code>hideResultsOnBlur</code>
          to prevent hiding typeahead's results until a user doesn't choose an item</p>
        `,
        component: require('!!raw-loader!./demos/show-on-blur/show-on-blur.ts'),
        html: require('!!raw-loader!./demos/show-on-blur/show-on-blur.html'),
        outlet: DemoTypeaheadShowOnBlurComponent
      },
      {
        title: 'Configuring defaults',
        anchor: 'configuration',
        component: require('!!raw-loader!./demos/config/config'),
        html: require('!!raw-loader!./demos/config/config.html'),
        outlet: DemoTypeaheadConfigComponent
      },
      {
        title: 'Is first item active',
        anchor: 'first-item-active',
        description: `
          <p>Use input property <code>typeaheadIsFirstItemActive</code> or config property <code>isFirstItemActive</code> to make the first item active/inactive </p>
        `,
        component: require('!!raw-loader!./demos/first-item-active/first-item-active.ts'),
        html: require('!!raw-loader!./demos/first-item-active/first-item-active.html'),
        outlet: DemoTypeaheadFirstItemActiveComponent
      },
      {
        title: 'Selected first item',
        anchor: 'selected-first-item',
        description: `
          <p>Use <code>typeaheadSelectFirstItem</code> property to make the first item in options list unselectable by tab and enter.</p>
        `,
        component: require('!!raw-loader!./demos/selected-first-item/selected-first-item.ts'),
        html: require('!!raw-loader!./demos/selected-first-item/selected-first-item.html'),
        outlet: DemotypeaheadSelectFirstItemComponent
      },
      {
        title: 'Order results',
        anchor: 'typeahead-ordering',
        description: `
          <p>Use <code>typeaheadOrderBy</code> property to order your result by a certain field and in certain direction</p>
        `,
        component: require('!!raw-loader!./demos/ordering/ordering.ts'),
        html: require('!!raw-loader!./demos/ordering/ordering.html'),
        outlet: DemoTypeaheadOrderingComponent
      },
      {
        title: 'Multiple search',
        anchor: 'multiple-search',
        component: require('!!raw-loader!./demos/multiple-search/multiple-search.ts'),
        html: require('!!raw-loader!./demos/multiple-search/multiple-search.html'),
        description: `
          <p>Set <code>typeaheadMultipleSearch</code> input property to <code>true</code>
          and provide the multiple search delimiter by <code>typeaheadMultipleSearchDelimiters</code>
          to be able to search typeahead again after using one of the provided delimiters. Default delimiter
          is "<code>,</code>" if <code>typeaheadMultipleSearchDelimiters</code> is not used.
          After picking a first value from typeahead
          dropdown, type "<code>,</code>" or "<code>|</code>" and then next value can be searched.
          This is demo with delimeters "<code>,</code>" and "<code>|</code>"</p>
        `,
        outlet: DemoTypeaheadMultipleSearchComponent
      }
    ]
  },
  {
    name: 'Installation',
    anchor: 'api-reference',
    tabName: 'api',
    outlet: ApiSectionsComponent,
    usage: require('!!raw-loader!./docs/usage.md'),
    importInfo: '<span class="pln">ng add ngx</span><span class="pun">-</span><span class="pln">bootstrap </span> --component <span class="pln">typeahead</span>',
    content: [
      {
        title: 'TypeaheadDirective',
        anchor: 'TypeaheadDirective',
        outlet: NgApiDocComponent
      },
      {
        title: 'TypeaheadConfig',
        anchor: 'bs-typeahead-config',
        outlet: NgApiDocConfigComponent
      },
      {
        title: 'TypeaheadOptionListContext',
        anchor: 'typeahead-option-list-context',
        outlet: NgApiDocConfigComponent
      },
      {
        title: 'TypeaheadOptionItemContext',
        anchor: 'typeahead-option-item-context',
        outlet: NgApiDocConfigComponent
      },
      {
        title: 'TypeaheadTemplateMethods',
        anchor: 'typeahead-template method',
        outlet: NgApiDocConfigComponent,
        showMethods: true
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
        title: 'Basic array',
        anchor: 'Basic-array-ex',
        outlet: DemoTypeaheadBasicComponent
      },
      {
        title: 'With animation',
        anchor: 'animated-ex',
        outlet: DemoTypeaheadAnimatedComponent
      },
      {
        title: 'Adaptive position',
        anchor: 'adaptive-position-ex',
        outlet: DemoTypeaheadAdaptivePositionComponent
      },
      {
        title: 'Item template',
        anchor: 'item-template-ex',
        outlet: DemoTypeaheadItemTemplateComponent
      },
      {
        title: 'List template',
        anchor: 'list-template-ex',
        outlet: DemoTypeaheadListTemplateComponent
      },
      {
        title: 'Option field',
        anchor: 'option-field-ex',
        outlet: DemoTypeaheadFieldComponent
      },
      {
        title: 'Async data',
        anchor: 'async-data-ex',
        outlet: DemoTypeaheadAsyncComponent
      },
      {
        title: 'Async using http request',
        anchor: 'async-http-request-ex',
        outlet: DemoTypeaheadAsyncHttpRequestComponent
      },
      {
        title: 'Cancel on focus lost',
        anchor: 'cancel-on-focus-lost-ex',
        outlet: DemoTypeaheadCancelRequestOnFocusLostComponent
      },
      {
        title: 'With delay',
        anchor: 'delay-ex',
        outlet: DemoTypeaheadDelayComponent
      },
      {
        title: 'Template-driven forms',
        anchor: 'forms-ex',
        outlet: DemoTypeaheadFormComponent
      },
      {
        title: 'Reactive forms',
        anchor: 'reactive-forms-ex',
        outlet: DemoTypeaheadReactiveFormComponent
      },
      {
        title: 'Grouping results',
        anchor: 'grouping-results-ex',
        outlet: DemoTypeaheadGroupingComponent
      },
      {
        title: 'Ignore spaces and order',
        anchor: 'single-world-ex',
        outlet: DemoTypeaheadSingleWorldComponent
      },
      {
        title: 'Phrase delimiters',
        anchor: 'phrase-delimiters-ex',
        outlet: DemoTypeaheadPhraseDelimitersComponent
      },
      {
        title: 'Dropup',
        anchor: 'dropup-ex',
        outlet: DemoTypeaheadDropupComponent
      },
      {
        title: 'On blur',
        anchor: 'on-blur-ex',
        outlet: DemoTypeaheadOnBlurComponent
      },
      {
        title: 'Append to body',
        anchor: 'container-ex',
        outlet: DemoTypeaheadContainerComponent
      },
      {
        title: 'No result',
        anchor: 'no-result-ex',
        outlet: DemoTypeaheadNoResultComponent
      },
      {
        title: 'Scrollable',
        anchor: 'scrollable-ex',
        outlet: DemoTypeaheadScrollableComponent
      },
      {
        title: 'Latinize',
        anchor: 'latinize-ex',
        outlet: DemoTypeaheadLatinizeComponent
      },
      {
        title: 'On select / On preview',
        anchor: 'on-select-ex',
        outlet: DemoTypeaheadOnSelectComponent
      },
      {
        title: 'Min length',
        anchor: 'min-length-ex',
        outlet: DemoTypeaheadMinLengthComponent
      },
      {
        title: 'Show results on blur',
        anchor: 'show-on-blur-ex',
        outlet: DemoTypeaheadShowOnBlurComponent
      },
      {
        title: 'Configuring defaults',
        anchor: 'configuration-ex',
        outlet: DemoTypeaheadConfigComponent
      },
      {
        title: 'Is first item active',
        anchor: 'first-item-active-ex',
        outlet: DemoTypeaheadFirstItemActiveComponent
      },
      {
        title: 'Selected first item',
        anchor: 'selected-first-item-ex',
        outlet: DemotypeaheadSelectFirstItemComponent
      },
      {
        title: 'Order results',
        anchor: 'typeahead-ordering-ex',
        outlet: DemoTypeaheadOrderingComponent
      },
      {
        title: 'Multiple search',
        anchor: 'multiple-search-ex',
        outlet: DemoTypeaheadMultipleSearchComponent
      }
    ]
  }
];
