import { NgTemplateOutlet } from '@angular/common';
import { afterNextRender, Component, computed, DestroyRef, inject, signal } from '@angular/core';
import { TuiFrame, TuiFrameBorderContent, TuiTerminal } from 'rng-tui';

const MOBILE_PAGE_COUNT = 2;

@Component({
  selector: 'app-root',
  imports: [NgTemplateOutlet, TuiTerminal, TuiFrame],
  templateUrl: './app.html',
  styleUrl: './app.scss'
})
export class App {
  private readonly destroyRef = inject(DestroyRef);

  protected readonly pageCount = MOBILE_PAGE_COUNT;
  protected readonly viewportWidth = signal(window.innerWidth);
  protected readonly isMobileLayout = computed(() => this.viewportWidth() < 768);
  protected readonly showSidePadding = computed(() => this.viewportWidth() >= 1200);
  protected readonly sidePaddingFillX = computed(() =>
    this.viewportWidth() > 1400 ? 2 : 1
  );

  protected readonly currentPageIndex = signal(0);
  protected readonly mobileFooter = computed((): TuiFrameBorderContent => ({
    bottom: {
      left: ` ${this.currentPageIndex() + 1}/${this.pageCount} `,
      center: ' ‹ tap sides to page › ',
      right: ' ',
    },
  }));

  protected copied = signal(false);

  constructor() {
    afterNextRender(() => {
      const onResize = () => {
        const wasMobile = this.isMobileLayout();
        this.viewportWidth.set(window.innerWidth);
        if (!wasMobile && this.isMobileLayout()) {
          this.currentPageIndex.set(0);
        }
      };
      window.addEventListener('resize', onResize);
      this.destroyRef.onDestroy(() => window.removeEventListener('resize', onResize));
    });
  }

  protected prevScreen(): void {
    const index = this.currentPageIndex();
    if (index > 0) {
      this.currentPageIndex.set(index - 1);
    }
  }

  protected nextScreen(): void {
    const index = this.currentPageIndex();
    if (index < this.pageCount - 1) {
      this.currentPageIndex.set(index + 1);
    }
  }

  async copySsh(): Promise<void> {
    try {
      await navigator.clipboard.writeText('ssh tui.rulof.dev');
      this.copied.set(true);
      setTimeout(() => this.copied.set(false), 2000);
    } catch {
      /* clipboard unavailable */
    }
  }
}
