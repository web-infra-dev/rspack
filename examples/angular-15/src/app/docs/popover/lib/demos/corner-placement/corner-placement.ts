import { Component } from '@angular/core';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-popover-corner-placement',
  templateUrl: './corner-placement.html'
})
export class DemoPopoverCornerPlacementComponent {
  placements = [
    'top left',
    'top right',
    'right top',
    'right bottom',
    'bottom right',
    'bottom left',
    'left bottom',
    'left top'
  ];
  placement: "top" | "bottom" | "left" | "right" | "auto" | "top left" | "top right" | "right top" | "right bottom" | "bottom right" | "bottom left" | "left bottom" | "left top" = 'top left';
}
