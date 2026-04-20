import { Directive, ElementRef, OnInit, inject } from '@angular/core';

@Directive({
  selector: '[appFadeIn]'
})
export class FadeInDirective implements OnInit {
  private el = inject(ElementRef);

  ngOnInit() {
    const el = this.el.nativeElement as HTMLElement;
    el.classList.add('fade-in');
    const obs = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          el.classList.add('visible');
          obs.disconnect();
        }
      },
      { threshold: 0.1 }
    );
    obs.observe(el);
  }
}
