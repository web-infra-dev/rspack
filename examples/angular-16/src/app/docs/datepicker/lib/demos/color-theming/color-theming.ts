import { Component } from '@angular/core';
import { BsDatepickerConfig } from 'ngx-bootstrap/datepicker';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-datepicker-color-theming',
  templateUrl: './color-theming.html'
})
export class DemoDatepickerColorThemingComponent {
  colorTheme = 'theme-green';

  bsConfig?: Partial<BsDatepickerConfig>;

  applyTheme(pop: any) {
    // create new object on each property change
    // so Angular can catch object reference change
    this.bsConfig = Object.assign({}, { containerClass: this.colorTheme });
    setTimeout(() => {
      pop.show();
    });
  }
}
