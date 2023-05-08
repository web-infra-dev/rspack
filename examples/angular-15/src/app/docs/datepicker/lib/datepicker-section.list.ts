import { ApiSectionsComponent } from '../../common-docs';
import { ContentSection } from '../../common-docs';
import { DemoDatepickerBasicComponent } from './demos/basic/basic';
import { DemoDatepickerByIsOpenPropComponent } from './demos/trigger-by-isopen-property/trigger-by-isopen-property';
import { DemoDatepickerChangeLocaleComponent } from './demos/change-locale/change-locale';
import { DemoDatepickerColorThemingComponent } from './demos/color-theming/color-theming';
import { DemoDatepickerConfigMethodComponent } from './demos/config-method/config-method';
import { DemoDatePickerConfigObjectComponent } from './demos/config-object/config-object';
import { DemoDatePickerCustomFormatComponent } from './demos/custom-format/custom-format';
import { DemoDatepickerDateInitialStateComponent } from './demos/date-initial-state/date-initial-state';
import { DemoDatepickerDatesDisabledComponent } from './demos/disable-dates/disable-dates';
import { DemoDatepickerDatesEnabledComponent } from './demos/enable-dates/enable-dates';
import { DemoDatepickerDaysDisabledComponent } from './demos/disable-days/disable-days';
import { DemoDatepickerDisabledComponent } from './demos/disabled/disabled.component';
import { DemoDatepickerFormsComponent } from './demos/forms/forms.component';
import { DemoDatepickerHideOnScrollComponent } from './demos/hide-on-scroll/hide-on-scroll';
import { DemoDatepickerInlineComponent } from './demos/inline-datepicker/inline-datepicker.component';
import { DemoDatepickerMinMaxComponent } from './demos/min-max/min-max.component';
import { DemoDatepickerMinModeComponent } from './demos/min-mode/min-mode.component';
import { DemoDatepickerOutsideClickComponent } from './demos/outside-click/outside-click';
import { DemoDatepickerPlacementComponent } from './demos/placement/placement';
import { DemoDatepickerReactiveFormsComponent } from './demos/reactive-forms/reactive-forms.component';
import { DemoDatePickerReturnFocusToInputComponent } from './demos/return-focus-to-input/return-focus-to-input.component';
import { DemoDatepickerDateCustomClassesComponent } from './demos/date-custom-classes/date-custom-classes';

import {
  DemoDatePickerSelectDatesFromOtherMonthsComponent
} from './demos/select-dates-from-other-months/select-dates-from-other-months';
import { DemoDatePickerAdaptivePositionComponent } from './demos/adaptive-position/adaptive-position';
import { DemoDatePickerAnimatedComponent } from './demos/animated/animated';
import { DemoDatepickerCustomTodayClassComponent } from './demos/custom-today-class/custom-today-class.component';
import { DemoDatePickerSelectWeekComponent } from './demos/select-week/select-week';
import { DemoDatepickerTriggersCustomComponent } from './demos/triggers-custom/triggers-custom';
import { DemoDatepickerTriggersManualComponent } from './demos/triggers-manual/triggers-manual';
import { DemoDatepickerValueChangeEventComponent } from './demos/value-change-event/value-change-event';
import { DemoDatePickerVisibilityEventsComponent } from './demos/visibility-events/visibility-events';
import { ExamplesComponent } from '../../common-docs';

import {
  NgApiDocComponent,
  NgApiDocConfigComponent
} from '../../common-docs';
import { DemoDatePickerQuickSelectRangesComponent } from './demos/quick-select-ranges/quick-select-ranges';
import { DemoDateRangePickerShowPreviousMonth } from './demos/daterangepicker-show-previous-month/show-previous-month';
import { DemoDatePickerSelectWeekRangeComponent } from './demos/select-week-range/select-week-range';
import { DemoDatePickerTooltipToSelectedDates } from './demos/tooltip-to-selected-dates/tooltip-to-selected-dates';
import { DemoDateRangePickerMaxDateRangeComponent } from './demos/max-date-range/max-date-range';
import { DemoDateRangePickerDisplayOneMonth } from './demos/daterangepicker-display-one-month/display-one-month';
import { DemoDatepickerTodayButtonComponent } from './demos/today-button/today-button';
import { DemoDatepickerClearButtonComponent } from './demos/clear-button/clear-button';
import { DemoDatepickerStartViewComponent } from "./demos/start-view/start-view";
import { DemoDatepickerPreventChangeToNextMonthComponent } from './demos/prevent-change-to-next-month/prevent-change-to-next-month.component';
import { DemoDatepickerWithTimepickerComponent } from './demos/with-timepicker/with-timepicker';
import { DatepickerCloseBehaviorComponent } from './demos/closeBehaviour/datepicker-close-behavior';
import { KeepDatesOutOfRulesComponent } from './demos/keep-dates-out-of-rules/keep-dates-out-of-rules.component';

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
        description: `
          <p>Note: If you installed ngx-bootstrap not via ng add command, you will need to perform a several actions</p>
          <p>Notable change is additional css for it <code> "/datepicker/bs-datepicker.css"</code> <br></p>
          <p>There are two ways of adding css:</p>
          <ul>
            <li>Load it from CDN. Add <code>&lt;link rel="stylesheet"
              href="https://unpkg.com/ngx-bootstrap/datepicker/bs-datepicker.css"&gt;</code> to your
              <code>index.html</code></li>
            <li>Load it from <code>dist/ngx-bootstrap/datepicker/bs-datepicker.css</code> via package bundler
              like Angular CLI, if you're using one.
            </li>
          </ul>
        `,
        outlet: DemoDatepickerBasicComponent
      },
      {
        title: 'Inline',
        anchor: 'inline-datepicker',
        component: require('!!raw-loader!./demos/inline-datepicker/inline-datepicker.component.ts'),
        html: require('!!raw-loader!./demos/inline-datepicker/inline-datepicker.component.html'),
        description: `<p>with initial state set by <code>bsInlineValue</code> property</p>`,
        outlet: DemoDatepickerInlineComponent
      },
      {
        title: 'With animation',
        anchor: 'animated',
        component: require('!!raw-loader!./demos/animated/animated'),
        html: require('!!raw-loader!./demos/animated/animated.html'),
        description: `You can enable animation via <code>isAnimated</code> config option`,
        outlet: DemoDatePickerAnimatedComponent
      },
      {
        title: 'Adaptive position',
        anchor: 'adaptive-position',
        component: require('!!raw-loader!./demos/adaptive-position/adaptive-position.ts'),
        html: require('!!raw-loader!./demos/adaptive-position/adaptive-position.html'),
        description: `<p>You can enable adaptive position via <code>adaptivePosition</code> option in <code>bsConfig</code></p>`,
        outlet: DemoDatePickerAdaptivePositionComponent
      },
      {
        title: 'Initial state',
        anchor: 'date-initial-state',
        component: require('!!raw-loader!./demos/date-initial-state/date-initial-state.ts'),
        html: require('!!raw-loader!./demos/date-initial-state/date-initial-state.html'),
        outlet: DemoDatepickerDateInitialStateComponent
      },
      {
        title: 'Custom date format',
        anchor: 'format',
        component: require('!!raw-loader!./demos/custom-format/custom-format.ts'),
        html: require('!!raw-loader!./demos/custom-format/custom-format.html'),
        description: `
          <p>You can easily change the date format by specifying the <code>dateInputFormat</code>
            in <code>[bsConfig]</code>
          </p>
          <p>To set your own date format you can use variety of formats provided by
          <a href="https://momentjs.com/docs/#/displaying/format/" target="_blank">moment.js</a></p>
          <p>The following examples show how to use several date formats inside a form:
            <ul>
              <li><code>YYYY-MM-DD</code></li>
              <li><code>MM/DD/YYYY</code></li>
              <li><code>MMMM Do YYYY,h:mm:ss a</code></li>
            </ul>
          </p>
        `,
        outlet: DemoDatePickerCustomFormatComponent
      },
      {
        title: 'Hide on scroll',
        anchor: 'hide-on-scroll',
        component: require('!!raw-loader!./demos/hide-on-scroll/hide-on-scroll.ts'),
        html: require('!!raw-loader!./demos/hide-on-scroll/hide-on-scroll.html'),
        description: `
          <p>Hide the datepicker on page scroll.</p>
        `,
        outlet: DemoDatepickerHideOnScrollComponent
      },
      {
        title: 'Themes',
        anchor: 'themes',
        component: require('!!raw-loader!./demos/color-theming/color-theming.ts'),
        html: require('!!raw-loader!./demos/color-theming/color-theming.html'),
        description: `
        <p>Datepicker comes with some default color schemes.
        You can change it by manipulating <code>containerClass</code> property in <code>bsConfig</code> object</p>
        <p>There are 6 color schemes: <code>theme-default</code>, <code>theme-green</code>, <code>theme-blue</code>,
        <code>theme-dark-blue</code>, <code>theme-red</code>, <code>theme-orange</code></p>
      `,
        outlet: DemoDatepickerColorThemingComponent
      },
      {
        title: 'Locales',
        anchor: 'locales',
        component: require('!!raw-loader!./demos/change-locale/change-locale.ts'),
        html: require('!!raw-loader!./demos/change-locale/change-locale.html'),
        description: `
          <p>Datepicker can use different locales. <br>It's possible to change a locale by calling
          <code>use</code>
          method of <code>BsLocaleService</code>, list of available locales is in dropdown below.</p>
          <p>To use a different locale, you have to import it from <code>ngx-bootstrap/chronos</code> first, then
          define it in your <code>@NgModule</code> using function <code>defineLocale</code></p>
          <p>Example: </p>
          <code>import { defineLocale } from 'ngx-bootstrap/chronos';</code><br>
          <code>import { deLocale } from 'ngx-bootstrap/locale';</code><br>
          <code>defineLocale('de', deLocale);</code>
          <br>
          <br>
        `,
        outlet: DemoDatepickerChangeLocaleComponent
      },
      {
        title: 'Min-max',
        anchor: 'min-max',
        component: require('!!raw-loader!./demos/min-max/min-max.component.ts'),
        html: require('!!raw-loader!./demos/min-max/min-max.component.html'),
        description: `
          <p>You can set min and max date of datepicker/daterangepicker using <code>minDate</code> and
          <code>maxDate</code> properties</p>
          <p>In the following example <code>minDate</code> is set to yesterday and <code>maxDate</code>
          to the current day in the next week</p>`,
        outlet: DemoDatepickerMinMaxComponent
      },
      {
        title: 'Days disabled',
        anchor: 'days-disabled',
        component: require('!!raw-loader!./demos/disable-days/disable-days.ts'),
        html: require('!!raw-loader!./demos/disable-days/disable-days.html'),
        description: `
          <p>You can set which days of the week should be disabled with <code>daysDisabled</code>
          <p>In the following example <code>daysDisabled</code> is set with an array which disabled Saturday and Sunday.
          Sunday is considered the first day of the week and thus has the value 0</p>`,
        outlet: DemoDatepickerDaysDisabledComponent
      },
      {
        title: 'Dates disabled',
        anchor: 'dates-disabled',
        component: require('!!raw-loader!./demos/disable-dates/disable-dates.ts'),
        html: require('!!raw-loader!./demos/disable-dates/disable-dates.html'),
        description: `
          <p>You can set which dates should be disabled with <code>datesDisabled</code></p>
          <p>In the following example <code>datesDisabled</code> is set with an array to disable 2020-02-05 and 2020-02-09.</p>
          <p>NOTE: DO NOT USE this functionality with <code>datesEnabled</code> at the same time</p>`,
        outlet: DemoDatepickerDatesDisabledComponent
      },
      {
        title: 'Dates enabled',
        anchor: 'dates-enabled',
        component: require('!!raw-loader!./demos/enable-dates/enable-dates.ts'),
        html: require('!!raw-loader!./demos/enable-dates/enable-dates.html'),
        description: `
          <p>You can set which dates should be enable with <code>datesEnabled</code></p>
          <p>In the following example <code>datesEnabled</code> is set with an array to enable 2020-02-06, 2020-02-08 and 2020-02-11. All other dates are disabled</p>
          <p>NOTE: DO NOT USE this functionality with <code>datesDisabled</code> at the same time</p>`,
        outlet: DemoDatepickerDatesEnabledComponent
      },
      {
        title: 'Display one month',
        anchor: 'display-one-month',
        component: require('!!raw-loader!./demos/daterangepicker-display-one-month/display-one-month.ts'),
        html: require('!!raw-loader!./demos/daterangepicker-display-one-month/display-one-month.html'),
        description: `<p>You can configure, how many months  you want to show for daterangepicker via <code>displayMonths</code> in <code>BsDaterangepickerConfig.</code></p>
        <p>With <code>displayOneMonthRange</code> you can show only one month for two cases</p>`,
        outlet: DemoDateRangePickerDisplayOneMonth
      },
      {
        title: 'Min-mode',
        anchor: 'min-mode',
        component: require('!!raw-loader!./demos/min-mode/min-mode.component.ts'),
        html: require('!!raw-loader!./demos/min-mode/min-mode.component.html'),
        description: `
          <p>You can set min view mode of datepicker using <code>minMode</code> property</p>
          <p>In the following example <code>minMode</code> is set to 'month'</p>`,
        outlet: DemoDatepickerMinModeComponent
      },
      {
        title: 'Disabled',
        anchor: 'disabled-datepicker',
        component: require('!!raw-loader!./demos/disabled/disabled.component.ts'),
        html: require('!!raw-loader!./demos/disabled/disabled.component.html'),
        description: `<p>If you want to disable datepicker's or daterangepicker's content set <code>isDisabled</code> property to true</p>`,
        outlet: DemoDatepickerDisabledComponent
      },
      {
        title: 'Custom today class',
        anchor: 'today-class',
        component: require('!!raw-loader!./demos/custom-today-class/custom-today-class.component.ts'),
        html: require('!!raw-loader!./demos/custom-today-class/custom-today-class.component.html'),
        description: `<p>If you want to add custom class to current day datepicker's content set value to <code>customTodayClass</code> option in <code>bsConfig</code></p>`,
        outlet: DemoDatepickerCustomTodayClassComponent
      },
      {
        title: 'Forms',
        anchor: 'forms',
        component: require('!!raw-loader!./demos/forms/forms.component.ts'),
        html: require('!!raw-loader!./demos/forms/forms.component.html'),
        description: `<p>Datepicker and daterangepicker can be used in forms. Keep in mind that
          value of <code>ngModel</code> is <code>Date</code> object for datepicker and array of 2
          <code>Date</code> objects for daterangepicker</p>`,
        outlet: DemoDatepickerFormsComponent
      },
      {
        title: 'Reactive forms',
        anchor: 'reactive-forms',
        component: require('!!raw-loader!./demos/reactive-forms/reactive-forms.component.ts'),
        html: require('!!raw-loader!./demos/reactive-forms/reactive-forms.component.html'),
        outlet: DemoDatepickerReactiveFormsComponent
      },
      {
        title: 'Return focus to input',
        anchor: 'return-focus-to-input',
        component: require('!!raw-loader!./demos/return-focus-to-input/return-focus-to-input.component.ts'),
        html: require('!!raw-loader!./demos/return-focus-to-input/return-focus-to-input.component.html'),
        description: `<p>Allows to return focus to input of datepicker or daterangepicker after the date or daterange selection</p>`,
        outlet: DemoDatePickerReturnFocusToInputComponent
      },
      {
        title: 'Manual triggering',
        anchor: 'triggers-manual',
        component: require('!!raw-loader!./demos/triggers-manual/triggers-manual.ts'),
        html: require('!!raw-loader!./demos/triggers-manual/triggers-manual.html'),
        description: `<p>You can manage datepicker's state by using its <code>show()</code>, <code>hide()</code>
          and <code>toggle()</code> methods</p>`,
        outlet: DemoDatepickerTriggersManualComponent
      },
      {
        title: 'Placement',
        anchor: 'placement',
        component: require('!!raw-loader!./demos/placement/placement.ts'),
        html: require('!!raw-loader!./demos/placement/placement.html'),
        description: `<p>Add <code>placement</code> property if you want to change placement</p>`,
        outlet: DemoDatepickerPlacementComponent
      },
      {
        title: 'Config method',
        anchor: 'config-method',
        component: require('!!raw-loader!./demos/config-method/config-method.ts'),
        html: require('!!raw-loader!./demos/config-method/config-method.html'),
        description: `<p>You can manage datepicker's options by using its <code>setConfig()</code> method</p>`,
        outlet: DemoDatepickerConfigMethodComponent
      },
      {
        title: 'Visibility Events',
        anchor: 'visibility-events',
        component: require('!!raw-loader!./demos/visibility-events/visibility-events.ts'),
        html: require('!!raw-loader!./demos/visibility-events/visibility-events.html'),
        description: `<p>You can subscribe to datepicker's visibility events</p>`,
        outlet: DemoDatePickerVisibilityEventsComponent
      },
      {
        title: 'Value change event',
        anchor: 'value-change-event',
        component: require('!!raw-loader!./demos/value-change-event/value-change-event.ts'),
        html: require('!!raw-loader!./demos/value-change-event/value-change-event.html'),
        description: `<p>You can subscribe to datepicker's value change event (<code>bsValueChange</code>).</p>`,
        outlet: DemoDatepickerValueChangeEventComponent
      },
      {
        title: 'Config properties',
        anchor: 'config-object',
        component: require('!!raw-loader!./demos/config-object/config-object.ts'),
        html: require('!!raw-loader!./demos/config-object/config-object.html'),
        description: `<p>You can configure the datepicker via its <code>bsConfig</code> option</p>`,
        outlet: DemoDatePickerConfigObjectComponent
      },
      {
        title: 'Select dates from other month',
        anchor: 'select-dates-from-other-month',
        component: require('!!raw-loader!./demos/select-dates-from-other-months/select-dates-from-other-months.ts'),
        html: require('!!raw-loader!./demos/select-dates-from-other-months/select-dates-from-other-months.html'),
        description: `<p>You can enable dates from other months via <code>selectFromOtherMonth</code> option in <code>bsConfig</code></p>`,
        outlet: DemoDatePickerSelectDatesFromOtherMonthsComponent
      },
      {
        title: 'Select week',
        anchor: 'select-week',
        component: require('!!raw-loader!./demos/select-week/select-week.ts'),
        html: require('!!raw-loader!./demos/select-week/select-week.html'),
        description: `<p>You can enable ability to select a week number (first day of the week will be selected) via <code>selectWeek</code> option in <code>bsConfig</code></p>`,
        outlet: DemoDatePickerSelectWeekComponent
      },
      {
        title: 'Select week range',
        anchor: 'select-week-range',
        component: require('!!raw-loader!./demos/select-week-range/select-week-range.ts'),
        html: require('!!raw-loader!./demos/select-week-range/select-week-range.html'),
        description: `<p>You can enable ability to select a week number (range with first weekday - last weekday will be selected) via <code>selectWeekRange</code> option in <code>bsConfig</code></p>`,
        outlet: DemoDatePickerSelectWeekRangeComponent
      },
      {
        title: 'Outside click',
        anchor: 'outside-click',
        component: require('!!raw-loader!./demos/outside-click/outside-click.ts'),
        html: require('!!raw-loader!./demos/outside-click/outside-click.html'),
        description: `<p>Datepicker closes after outside click by default. To change
          this behavior, use <code>outsideClick</code> property.</p>`,
        outlet: DemoDatepickerOutsideClickComponent
      },
      {
        title: 'Trigger by isOpen property',
        anchor: 'trigger-by-isopen-property',
        component: require('!!raw-loader!./demos/trigger-by-isopen-property/trigger-by-isopen-property.ts'),
        html: require('!!raw-loader!./demos/trigger-by-isopen-property/trigger-by-isopen-property.html'),
        description: `<p>Datepicker can be shown or hidden by changing <code>isOpen</code> property</p>`,
        outlet: DemoDatepickerByIsOpenPropComponent
      },
      {
        title: 'Custom triggers',
        anchor: 'triggers-custom',
        component: require('!!raw-loader!./demos/triggers-custom/triggers-custom.ts'),
        html: require('!!raw-loader!./demos/triggers-custom/triggers-custom.html'),
        description: `<p>Use different triggers ( for example <code>keydown</code>, <code>mouseenter</code> and
          <code>dblclick</code> ) to interact with datepicker</p>`,
        outlet: DemoDatepickerTriggersCustomComponent
      },
      {
        title: 'Date custom classes',
        anchor: 'date-custom-classes',
        component: require('!!raw-loader!./demos/date-custom-classes/date-custom-classes.ts'),
        html: require('!!raw-loader!./demos/date-custom-classes/date-custom-classes.html'),
        style: require('!!raw-loader!./demos/date-custom-classes/date-custom-classes.scss'),
        description: `<p>Style dates with custom classes</p>`,
        outlet: DemoDatepickerDateCustomClassesComponent
      },
      {
        title: 'Tooltip for selected dates',
        anchor: 'tooltip-for-selected-dates',
        component: require('!!raw-loader!./demos/tooltip-to-selected-dates/tooltip-to-selected-dates.ts'),
        html: require('!!raw-loader!./demos/tooltip-to-selected-dates/tooltip-to-selected-dates.html'),
        description: ``,
        outlet: DemoDatePickerTooltipToSelectedDates
      },
      {
        title: 'Quick select ranges',
        anchor: 'quick-select-ranges',
        component: require('!!raw-loader!./demos/quick-select-ranges/quick-select-ranges.ts'),
        html: require('!!raw-loader!./demos/quick-select-ranges/quick-select-ranges.html'),
        description: `<p>Quick select ranges can be added to Daterangepicker using <code>ranges</code></p>`,
        outlet: DemoDatePickerQuickSelectRangesComponent
      },
      {
        title: 'Prevent change to next month',
        anchor: 'prevent-change-to-next-month',
        component: require('!!raw-loader!./demos/prevent-change-to-next-month/prevent-change-to-next-month.component.ts'),
        html: require('!!raw-loader!./demos/prevent-change-to-next-month/prevent-change-to-next-month.component.html'),
        description: `<p>Pick some date from second month and it wont change to the next month</p>`,
        outlet: DemoDatepickerPreventChangeToNextMonthComponent
      },
      {
        title: 'Previous month in Daterangepicker',
        anchor: 'daterangepicker-previous-month',
        component: require('!!raw-loader!./demos/daterangepicker-show-previous-month/show-previous-month.ts'),
        html: require('!!raw-loader!./demos/daterangepicker-show-previous-month/show-previous-month.html'),
        description: `<p>Pick previous & current month instead of current & next month.When daterange selected and related to current month,
        daterangepicker will works by default, with current & next month</p>`,
        outlet: DemoDateRangePickerShowPreviousMonth
      },
      {
        title: 'Show Today Button',
        anchor: 'datepicker-show-today-button',
        component: require('!!raw-loader!./demos/today-button/today-button.ts'),
        html: require('!!raw-loader!./demos/today-button/today-button.html'),
        description: `<p>Display an optional 'Today' button that will automatically select today's date.</p>`,
        outlet: DemoDatepickerTodayButtonComponent,
      },
      {
        title: 'Show Clear Button',
        anchor: 'datepicker-show-clear-button',
        component: require('!!raw-loader!./demos/clear-button/clear-button.ts'),
        html: require('!!raw-loader!./demos/clear-button/clear-button.html'),
        description: `<p>Display an optional 'Clear' button that will automatically clear date.</p>`,
        outlet: DemoDatepickerClearButtonComponent
      },
      {
        title: 'Start view',
        anchor: 'start-view',
        component: require('!!raw-loader!./demos/start-view/start-view.ts'),
        html: require('!!raw-loader!./demos/start-view/start-view.html'),
        description: `<p>Add <code>startView</code> property if you want to change start view</p>`,
        outlet: DemoDatepickerStartViewComponent
      },
      {
        title: 'Max Date Range in Daterangepicker',
        anchor: 'daterangepicker-max-date-range',
        component: require('!!raw-loader!./demos/max-date-range/max-date-range.ts'),
        html: require('!!raw-loader!./demos/max-date-range/max-date-range.html'),
        description: `<p>Max date range after first date selection can be added to Daterangepicker using <code>maxDateRange</code>.</p>
        <p>If you also use <code>maxDate</code> property, you can't select second date, which exceeds value of <code>maxDate</code>.</p>`,
        outlet: DemoDateRangePickerMaxDateRangeComponent
      },
      {
        title: 'With timepicker',
        anchor: 'with-timepicker',
        component: require('!!raw-loader!./demos/with-timepicker/with-timepicker'),
        html: require('!!raw-loader!./demos/with-timepicker/with-timepicker.html'),
        description: `You can enable timepicker via <code>withTimepicker</code> config option`,
        outlet: DemoDatepickerWithTimepickerComponent
      },
      {
        title: 'Close behavior with timepicker changes',
        anchor: 'close-behavior',
        component: require('!!raw-loader!./demos/closeBehaviour/datepicker-close-behavior'),
        html: require('!!raw-loader!./demos/closeBehaviour/datepicker-close-behavior.html'),
        description: `If you use datepicker with timepicker together, you are able to set <code>keepDatepickerOpened</code> config option and keep datepicker opened until date isn't changed`,
        outlet: DatepickerCloseBehaviorComponent
      },
      {
        title: "Don't overwrite dates out of rule",
        anchor: 'keep-dates-out-of-rules',
        component: require('!!raw-loader!./demos/keep-dates-out-of-rules/keep-dates-out-of-rules.component'),
        html: require('!!raw-loader!./demos/keep-dates-out-of-rules/keep-dates-out-of-rules.component.html'),
        description: `<p>If you use datepicker with rules (minDate, maxDate) you can set config property <code>keepDatesOutOfRules</code> to true to avoid overwriting invalid dates. Default value is false.</p>`,
        outlet: KeepDatesOutOfRulesComponent
      }
    ]
  },
  {
    name: 'Installation',
    anchor: 'api-reference',
    tabName: 'api',
    usage: require('!!raw-loader!./docs/usage.md'),
    importInfo: '<span class="pln">ng add ngx</span><span class="pun">-</span><span class="pln">bootstrap </span> --component <span class="pln">datepicker</span>',
    outlet: ApiSectionsComponent,
    content: [
      {
        title: 'BsDatepickerDirective',
        anchor: 'bs-datepicker-component',
        outlet: NgApiDocComponent
      },
      {
        title: 'BsDaterangepickerDirective',
        anchor: 'bs-daterangepicker',
        outlet: NgApiDocComponent
      },
      {
        title: 'BsDatepickerInlineDirective',
        anchor: 'bs-datepicker-inline',
        outlet: NgApiDocComponent
      },
      {
        title: 'BsDatepickerConfig',
        anchor: 'bs-datepicker-config',
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
        outlet: DemoDatepickerBasicComponent
      },
      {
        title: 'Inline',
        anchor: 'inline-datepicker-ex',
        outlet: DemoDatepickerInlineComponent
      },
      {
        title: 'With animation',
        anchor: 'animated-ex',
        outlet: DemoDatePickerAnimatedComponent
      },
      {
        title: 'Adaptive position',
        anchor: 'adaptive-position-ex',
        outlet: DemoDatePickerAdaptivePositionComponent
      },
      {
        title: 'Initial state',
        anchor: 'date-initial-state-ex',
        outlet: DemoDatepickerDateInitialStateComponent
      },
      {
        title: 'Custom date format',
        anchor: 'format-ex',
        outlet: DemoDatePickerCustomFormatComponent
      },
      {
        title: 'Hide on scroll',
        anchor: 'hide-on-scroll-ex',
        outlet: DemoDatepickerHideOnScrollComponent
      },
      {
        title: 'Themes',
        anchor: 'themes-ex',
        outlet: DemoDatepickerColorThemingComponent
      },
      {
        title: 'Locales',
        anchor: 'locales-ex',
        outlet: DemoDatepickerChangeLocaleComponent
      },
      {
        title: 'Min-max',
        anchor: 'min-max-ex',
        outlet: DemoDatepickerMinMaxComponent
      },
      {
        title: 'Days disabled',
        anchor: 'days-disabled-ex',
        outlet: DemoDatepickerDaysDisabledComponent
      },
      {
        title: 'Dates disabled',
        anchor: 'dates-disabled-ex',
        outlet: DemoDatepickerDatesDisabledComponent
      },
      {
        title: 'Dates enabled',
        anchor: 'dates-enabled-ex',
        outlet: DemoDatepickerDatesEnabledComponent
      },
      {
        title: 'Display one month',
        anchor: 'display-one-month-ex',
        outlet: DemoDateRangePickerDisplayOneMonth
      },
      {
        title: 'Min-mode',
        anchor: 'min-mode-ex',
        outlet: DemoDatepickerMinModeComponent
      },
      {
        title: 'Disabled',
        anchor: 'disabled-datepicker-ex',
        outlet: DemoDatepickerDisabledComponent
      },
      {
        title: 'Custom today class',
        anchor: 'today-class-ex',
        outlet: DemoDatepickerCustomTodayClassComponent
      },
      {
        title: 'Forms',
        anchor: 'forms-ex',
        outlet: DemoDatepickerFormsComponent
      },
      {
        title: 'Reactive forms',
        anchor: 'reactive-forms-ex',
        outlet: DemoDatepickerReactiveFormsComponent
      },
      {
        title: 'Return focus to input',
        anchor: 'return-focus-to-input-ex',
        outlet: DemoDatePickerReturnFocusToInputComponent
      },
      {
        title: 'Manual triggering',
        anchor: 'triggers-manual-ex',
        outlet: DemoDatepickerTriggersManualComponent
      },
      {
        title: 'Placement',
        anchor: 'placement-ex',
        outlet: DemoDatepickerPlacementComponent
      },
      {
        title: 'Config method',
        anchor: 'config-method-ex',
        outlet: DemoDatepickerConfigMethodComponent
      },
      {
        title: 'Visibility Events',
        anchor: 'visibility-events-ex',
        outlet: DemoDatePickerVisibilityEventsComponent
      },
      {
        title: 'Value change event',
        anchor: 'value-change-event-ex',
        outlet: DemoDatepickerValueChangeEventComponent
      },
      {
        title: 'Config properties',
        anchor: 'config-object-ex',
        outlet: DemoDatePickerConfigObjectComponent
      },
      {
        title: 'Select dates from other month',
        anchor: 'select-dates-from-other-month-ex',
        outlet: DemoDatePickerSelectDatesFromOtherMonthsComponent
      },
      {
        title: 'Select week',
        anchor: 'select-week-ex',
        outlet: DemoDatePickerSelectWeekComponent
      },
      {
        title: 'Select week range',
        anchor: 'select-week-range-ex',
        outlet: DemoDatePickerSelectWeekRangeComponent
      },
      {
        title: 'Outside click',
        anchor: 'outside-click-ex',
        outlet: DemoDatepickerOutsideClickComponent
      },
      {
        title: 'Trigger by isOpen property',
        anchor: 'trigger-by-isopen-property-ex',
        outlet: DemoDatepickerByIsOpenPropComponent
      },
      {
        title: 'Custom triggers',
        anchor: 'triggers-custom-ex',
        outlet: DemoDatepickerTriggersCustomComponent
      },
      {
        title: 'Date custom classes',
        anchor: 'date-custom-classes-ex',
        outlet: DemoDatepickerDateCustomClassesComponent
      },
      {
        title: 'Tooltip for selected dates',
        anchor: 'tooltip-for-selected-dates-ex',
        outlet: DemoDatePickerTooltipToSelectedDates
      },
      {
        title: 'Quick select ranges',
        anchor: 'quick-select-ranges-ex',
        outlet: DemoDatePickerQuickSelectRangesComponent
      },
      {
        title: 'Prevent change to next month',
        anchor: 'prevent-change-to-next-month-ex',
        outlet: DemoDatepickerPreventChangeToNextMonthComponent
      },
      {
        title: 'Previous month in Daterangepicker',
        anchor: 'daterangepicker-previous-month-ex',
        outlet: DemoDateRangePickerShowPreviousMonth
      },
      {
        title: 'Show Today Button',
        anchor: 'datepicker-show-today-button-ex',
        outlet: DemoDatepickerTodayButtonComponent,
      },
      {
        title: 'Show Clear Button',
        anchor: 'datepicker-show-clear-button-ex',
        outlet: DemoDatepickerClearButtonComponent
      },
      {
        title: 'Start view',
        anchor: 'start-view-ex',
        outlet: DemoDatepickerStartViewComponent
      },
      {
        title: 'Max Date Range in Daterangepicker',
        anchor: 'daterangepicker-max-date-range-ex',
        outlet: DemoDateRangePickerMaxDateRangeComponent
      },
      {
        title: 'With timepicker',
        anchor: 'with-timepicker-ex',
        outlet: DemoDatepickerWithTimepickerComponent
      },
      {
        title: 'datepicker close behavior with timepicker',
        anchor: 'close-behavior',
        outlet: DatepickerCloseBehaviorComponent
      },
      {
        title: "Don't overwrite dates out of rule",
        anchor: 'keep-dates-out-of-rules',
        outlet: KeepDatesOutOfRulesComponent
      },
    ]
  }
];
