import { Component, HostListener, ViewChild } from '@angular/core';
import { BsDatepickerDirective } from 'ngx-bootstrap/datepicker';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-date-picker-hide-on-scroll',
  templateUrl: './hide-on-scroll.html'
})
export class DemoDatepickerHideOnScrollComponent {
  @ViewChild(BsDatepickerDirective, { static: false }) datepicker?: BsDatepickerDirective;

  @HostListener('window:scroll')
  onScrollEvent() {
    this.datepicker?.hide();
  }
}
