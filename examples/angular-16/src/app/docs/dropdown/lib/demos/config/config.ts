import { Component } from '@angular/core';
import { BsDropdownConfig } from 'ngx-bootstrap/dropdown';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-dropdown-config',
  templateUrl: './config.html',
  providers: [{ provide: BsDropdownConfig, useValue: { autoClose: false } }]
})
export class DemoDropdownConfigComponent {}
