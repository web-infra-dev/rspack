import { Component, ViewChild } from '@angular/core';
import { BsDaterangepickerDirective, BsDatepickerConfig } from 'ngx-bootstrap/datepicker';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-datepicker-config-method',
  templateUrl: './config-method.html'
})
export class DemoDatepickerConfigMethodComponent {
  @ViewChild('dp', { static: false }) datepicker?: BsDaterangepickerDirective;

  bsConfig?: Partial<BsDatepickerConfig>;
  minDate = new Date(2018, 5, 13);

  setOptions(): void {
    this.bsConfig = Object.assign({}, { minDate: this.minDate });
    this.datepicker?.setConfig();

    setTimeout(() => {
      this.datepicker?.toggle();
    });
  }
}
