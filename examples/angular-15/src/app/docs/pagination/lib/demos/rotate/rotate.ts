import { Component } from '@angular/core';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-pagination-rotate',
  templateUrl: './rotate.html'
})
export class DemoPaginationRotateComponent {
  rotate = true;
  maxSize = 5;
  status = "ON";

  toggleBtn() : void {
    this.rotate = !this.rotate;
    this.status = this.rotate ? "ON" : "OFF";
  }
}
