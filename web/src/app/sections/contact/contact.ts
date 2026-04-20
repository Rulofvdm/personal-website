import { Component, signal } from '@angular/core';
import { FadeInDirective } from '../../shared/fade-in.directive';

@Component({
  selector: 'app-contact',
  imports: [FadeInDirective],
  templateUrl: './contact.html',
  styleUrl: './contact.scss'
})
export class ContactComponent {
  protected copied = signal(false);

  async copySSH() {
    try {
      await navigator.clipboard.writeText('ssh rulof.dev');
      this.copied.set(true);
      setTimeout(() => this.copied.set(false), 2000);
    } catch {}
  }
}
