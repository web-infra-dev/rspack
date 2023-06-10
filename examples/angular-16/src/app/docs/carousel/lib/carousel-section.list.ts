import { DemoCarouseBasicComponent } from './demos/basic/basic';
import { DemoCarouselCaptionsComponent } from './demos/captions/captions';
import { DemoCarouselConfigComponent } from './demos/config/config';
import { DemoCarouselDynamicComponent } from './demos/dynamic/dynamic';
import { DemoCarouselPauseOnHoverComponent } from './demos/pause-on-hover/pause-on-hover';
import { DemoCarouselCustomContentComponent } from './demos/custom-content/custom-content';
import { DemoCarouselIntervalComponent } from './demos/interval/interval';
import { DemoCarouselDisableIndicatorComponent } from './demos/disable-indicator/disable-indicator';
import { DemoCarouselDisableLoopingComponent } from './demos/disable-looping/disable-looping';
import { DemoCarouselSlideChangedEventComponent } from './demos/slide-changed-event/slide-changed-event';
import { DemoCarouselMultilistComponent } from './demos/multilist/multilist';
import { DemoCarouselMultilistSingleOffsetComponent } from './demos/multilist-single-offset/multilist-single-offset';
import { DemoCarouselMultilistFromIndexComponent } from './demos/multilist-from-index/multilist-from-index';
import { DemoCarouselMultilistIndicatorsByChunkComponent } from './demos/multilist-indicators-by-chunk/multilist-indicators-by-chunk';
import { DemoAccessibilityComponent } from './demos/accessibility/accessibility';

import { ContentSection } from '../../common-docs';
import { ExamplesComponent } from '../../common-docs';
import { ApiSectionsComponent } from '../../common-docs';

import { NgApiDocComponent, NgApiDocConfigComponent } from '../../common-docs';
import { DemoCarouselPauseOnFocusComponent } from './demos/pause-on-focus/pause-on-focus';
import { DemoCarouseAnimatedComponent } from './demos/animated/animated';

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
        outlet: DemoCarouseBasicComponent
      },
      {
        title: 'Optional captions',
        anchor: 'captions',
        description: `<p>Add captions to your slides easily with the <code>.carousel-caption</code>
          element within any <code>&lt;slide></code>. Place just about any optional HTML within there
          and it will be automatically aligned and formatted.</p>`,
        component: require('!!raw-loader!./demos/captions/captions.ts'),
        html: require('!!raw-loader!./demos/captions/captions.html'),
        outlet: DemoCarouselCaptionsComponent
      },
      {
        title: 'Configuring defaults',
        anchor: 'config-defaults',
        component: require('!!raw-loader!./demos/config/config.ts'),
        html: require('!!raw-loader!./demos/config/config.html'),
        outlet: DemoCarouselConfigComponent
      },
      {
        title: 'Dynamic Slides',
        anchor: 'dynamic-slides',
        component: require('!!raw-loader!./demos/dynamic/dynamic.ts'),
        html: require('!!raw-loader!./demos/dynamic/dynamic.html'),
        outlet: DemoCarouselDynamicComponent
      },
      {
        title: 'Pause on hover',
        anchor: 'pause-on-hover',
        description: `<p>If <code>noPause</code> is set to <code>false</code>
          , carousel autoplay will be stopped when carousel receives hover.</p>`,
        component: require('!!raw-loader!./demos/pause-on-hover/pause-on-hover.ts'),
        html: require('!!raw-loader!./demos/pause-on-hover/pause-on-hover.html'),
        outlet: DemoCarouselPauseOnHoverComponent
      },
      {
        title: 'Pause on focus',
        anchor: 'pause-on-focus',
        description: `<p>If <code>pauseOnFocus</code> is set to <code>true</code>
          , carousel autoplay will be stopped when carousel receives focus.</p>`,
        component: require('!!raw-loader!./demos/pause-on-focus/pause-on-focus.ts'),
        html: require('!!raw-loader!./demos/pause-on-focus/pause-on-focus.html'),
        outlet: DemoCarouselPauseOnFocusComponent
      },
      {
        title: 'Custom content',
        anchor: 'custom-content',
        component: require('!!raw-loader!./demos/custom-content/custom-content.ts'),
        html: require('!!raw-loader!./demos/custom-content/custom-content.html'),
        outlet: DemoCarouselCustomContentComponent
      },
      {
        title: 'Disable slide looping',
        anchor: 'disable-looping',
        component: require('!!raw-loader!./demos/disable-looping/disable-looping.ts'),
        html: require('!!raw-loader!./demos/disable-looping/disable-looping.html'),
        outlet: DemoCarouselDisableLoopingComponent
      },
      {
        title: 'Disable indicator',
        anchor: 'disable-indicator',
        component: require('!!raw-loader!./demos/disable-indicator/disable-indicator.ts'),
        html: require('!!raw-loader!./demos/disable-indicator/disable-indicator.html'),
        outlet: DemoCarouselDisableIndicatorComponent
      },
      {
        title: 'Interval',
        anchor: 'slides-interval',
        component: require('!!raw-loader!./demos/interval/interval.ts'),
        html: require('!!raw-loader!./demos/interval/interval.html'),
        outlet: DemoCarouselIntervalComponent
      },
      {
        title: 'Slide changed event',
        anchor: 'slide-changed-event',
        component: require('!!raw-loader!./demos/slide-changed-event/slide-changed-event.ts'),
        html: require('!!raw-loader!./demos/slide-changed-event/slide-changed-event.html'),
        outlet: DemoCarouselSlideChangedEventComponent
      },
      {
        title: 'Multilist',
        anchor: 'multilist',
        component: require('!!raw-loader!./demos/multilist/multilist.ts'),
        html: require('!!raw-loader!./demos/multilist/multilist.html'),
        outlet: DemoCarouselMultilistComponent
      },
      {
        title: 'Multilist Single Offset',
        anchor: 'multilist-single-offset',
        component: require('!!raw-loader!./demos/multilist-single-offset/multilist-single-offset.ts'),
        html: require('!!raw-loader!./demos/multilist-single-offset/multilist-single-offset.html'),
        outlet: DemoCarouselMultilistSingleOffsetComponent
      },
      {
        title: 'Multilist Start From Index',
        anchor: 'multilist-from-index',
        component: require('!!raw-loader!./demos/multilist-from-index/multilist-from-index.ts'),
        html: require('!!raw-loader!./demos/multilist-from-index/multilist-from-index.html'),
        outlet: DemoCarouselMultilistFromIndexComponent
      },
      {
        title: 'Multilist Indicators By Chunk',
        anchor: 'multilist-indicators-by-chunk',
        component: require('!!raw-loader!./demos/multilist-indicators-by-chunk/multilist-indicators-by-chunk.ts'),
        html: require('!!raw-loader!./demos/multilist-indicators-by-chunk/multilist-indicators-by-chunk.html'),
        outlet: DemoCarouselMultilistIndicatorsByChunkComponent
      },
      {
        title: 'With animation',
        anchor: 'animated',
        component: require('!!raw-loader!./demos/animated/animated.ts'),
        html: require('!!raw-loader!./demos/animated/animated.html'),
        outlet: DemoCarouseAnimatedComponent
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
    importInfo: '<span class="pln">ng add ngx</span><span class="pun">-</span><span class="pln">bootstrap </span> --component <span class="pln">carousel</span>',
    outlet: ApiSectionsComponent,
    content: [
      {
        title: 'CarouselComponent',
        anchor: 'carousel-component',
        outlet: NgApiDocComponent
      },
      {
        title: 'SlideComponent',
        anchor: 'slide-component',
        outlet: NgApiDocComponent
      },
      {
        title: 'CarouselConfig',
        anchor: 'carousel-config',
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
        outlet: DemoCarouseBasicComponent
      },
      {
        title: 'Optional captions',
        anchor: 'captions-ex',
        outlet: DemoCarouselCaptionsComponent
      },
      {
        title: 'Configuring defaults',
        anchor: 'config-defaults-ex',
        outlet: DemoCarouselConfigComponent
      },
      {
        title: 'Dynamic Slides',
        anchor: 'dynamic-slides-ex',
        outlet: DemoCarouselDynamicComponent
      },
      {
        title: 'Pause on hover',
        anchor: 'pause-on-hover-ex',
        outlet: DemoCarouselPauseOnHoverComponent
      },
      {
        title: 'Pause on focus',
        anchor: 'pause-on-focus-ex',
        outlet: DemoCarouselPauseOnFocusComponent
      },
      {
        title: 'Custom content',
        anchor: 'custom-content-ex',
        outlet: DemoCarouselCustomContentComponent
      },
      {
        title: 'Disable slide looping',
        anchor: 'disable-looping-ex',
        outlet: DemoCarouselDisableLoopingComponent
      },
      {
        title: 'Disable indicator',
        anchor: 'disable-indicator-ex',
        outlet: DemoCarouselDisableIndicatorComponent
      },
      {
        title: 'Interval',
        anchor: 'slides-interval-ex',
        outlet: DemoCarouselIntervalComponent
      },
      {
        title: 'Slide changed event',
        anchor: 'slide-changed-event-ex',
        outlet: DemoCarouselSlideChangedEventComponent
      },
      {
        title: 'Multilist',
        anchor: 'multilist-ex',
        outlet: DemoCarouselMultilistComponent
      },
      {
        title: 'Multilist Single Offset',
        anchor: 'multilist-single-offset-ex',
        outlet: DemoCarouselMultilistSingleOffsetComponent
      },
      {
        title: 'Multilist Start From Index',
        anchor: 'multilist-from-index-ex',
        outlet: DemoCarouselMultilistFromIndexComponent
      },
      {
        title: 'Multilist Indicators By Chunk',
        anchor: 'multilist-indicators-by-chunk-ex',
        outlet: DemoCarouselMultilistIndicatorsByChunkComponent
      },
      {
        title: 'With animation',
        anchor: 'animated-ex',
        outlet: DemoCarouseAnimatedComponent
      }
    ]
  }
];
