import { Component, OnInit } from '@angular/core';

import { BsDatepickerConfig, BsDatepickerViewMode } from 'ngx-bootstrap/datepicker';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-datepicker-min-mode',
  templateUrl: './min-mode.component.html'
})
export class DemoDatepickerMinModeComponent implements OnInit {
  datePickerValue: Date = new Date(2020, 7);
  dateRangePickerValue?: (Date | undefined)[];
  range1: Date = new Date(2020, 5);
  range2: Date = new Date(2020, 8);
  minMode: BsDatepickerViewMode = 'month';

  bsConfig?: Partial<BsDatepickerConfig>;

  ngOnInit(): void {
    this.dateRangePickerValue = [this.range1, this.range2];
    this.bsConfig = Object.assign({}, {
      minMode : this.minMode
    });
  }
}
