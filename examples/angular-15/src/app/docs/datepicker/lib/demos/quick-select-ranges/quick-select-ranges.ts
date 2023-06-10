import { Component } from '@angular/core';

interface IRange {
  value: Date[];
  label: string;
}

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-datepicker-quick-select-ranges',
  templateUrl: './quick-select-ranges.html'
})
export class DemoDatePickerQuickSelectRangesComponent {
  ranges: IRange[] = [{
    value: [new Date(new Date().setDate(new Date().getDate() - 7)), new Date()],
    label: 'Last 7 Days'
  }, {
    value: [new Date(), new Date(new Date().setDate(new Date().getDate() + 7))],
    label: 'Next 7 Days'
  }];

}
