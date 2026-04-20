import { Component, OnInit, OnDestroy, signal } from '@angular/core';
import { CursorComponent } from '../../shared/cursor/cursor';

@Component({
  selector: 'app-hero',
  imports: [CursorComponent],
  templateUrl: './hero.html',
  styleUrl: './hero.scss'
})
export class HeroComponent implements OnInit, OnDestroy {
  private readonly stack = ['Angular', 'TypeScript', 'Node.js', 'GraphQL', 'Rust (learning)'];
  protected current = signal('');
  private stackIdx = 0;
  private charIdx = 0;
  private deleting = false;
  private timer: ReturnType<typeof setTimeout> | null = null;

  ngOnInit() {
    this.tick();
  }

  ngOnDestroy() {
    if (this.timer) clearTimeout(this.timer);
  }

  private tick() {
    const word = this.stack[this.stackIdx];

    if (!this.deleting) {
      this.charIdx++;
      this.current.set(word.slice(0, this.charIdx));
      if (this.charIdx === word.length) {
        this.deleting = true;
        this.timer = setTimeout(() => this.tick(), 1800);
        return;
      }
    } else {
      this.charIdx--;
      this.current.set(word.slice(0, this.charIdx));
      if (this.charIdx === 0) {
        this.deleting = false;
        this.stackIdx = (this.stackIdx + 1) % this.stack.length;
      }
    }

    this.timer = setTimeout(() => this.tick(), this.deleting ? 55 : 95);
  }
}
