import { Component } from '@angular/core';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-carousel-dynamic',
  templateUrl: './dynamic.html'
})
export class DemoCarouselDynamicComponent {
  slides: { image: string; text?: string }[] = [];
  activeSlideIndex = 0;

  constructor() {
    for (let i = 0; i < 4; i++) {
      this.addSlide();
    }
  }

  addSlide(): void {
    this.slides.push({
      image: `assets/images/nature/${this.slides.length % 8 + 1}.jpg`
    });
  }

  removeSlide(index?: number): void {
    const toRemove = index ? index : this.activeSlideIndex;
    this.slides.splice(toRemove, 1);
  }
}
