# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

---

## Project Overview

Personal website for **Rulof van der Merwe** at `rulof.dev`. Monorepo with two sub-projects:

```
rulof.dev/
├── web/     # Angular app → Cloudflare Pages
└── tui/     # Rust SSH TUI → Hetzner VPS
```

Build `web/` first. `tui/` is phase 2.

---

## `web/` — Angular App

### Dev Environment

Enter the Nix dev shell first:
```bash
nix develop
```
Provides: Node.js 22, Angular CLI, TypeScript, npm.

### Common Commands

```bash
ng serve                                        # dev server
ng build --configuration production             # production build
ng test                                         # unit tests
ng test --include="**/foo.spec.ts"              # single test file
ng lint                                         # lint
```

Build output: `web/dist/rulof/browser/` → deployed to Cloudflare Pages.

### Tech Decisions

- Angular latest stable, **standalone components** only (no NgModules)
- TypeScript **strict mode**
- SCSS for styling
- No SSR. No backend. Purely static.
- No heavy animation libraries — IntersectionObserver + CSS transitions only
- No unnecessary dependencies — must score 95+ Lighthouse

### Architecture

```
web/src/
├── app/
│   ├── app.component.ts        # root standalone component, single scrollable page
│   ├── sections/               # one component per section (hero, about, experience, skills, projects, contact)
│   └── shared/cursor/          # blinking cursor component (CSS only)
├── styles/
│   ├── _variables.scss         # colours, fonts, spacing tokens
│   ├── _typography.scss
│   ├── _reset.scss
│   └── styles.scss
└── index.html
```

Single page, no routing.

### Design Constraints

- Dark background: `#0d0d0d` or `#111`
- Monospace primary font: JetBrains Mono (or Berkeley Mono / Iosevka / Commit Mono)
- Single accent colour — pick one and commit (amber `#f5a623`, electric green `#39ff14`, or desaturated teal)
- No gradients. No cards. No box shadows everywhere.
- Blinking cursor in hero (CSS only)
- SSH Easter egg at bottom: `$ ssh rulof.dev` — clicking copies to clipboard, shows brief "copied" confirmation

### Content Source of Truth

See the spec document. Key copy:
- Hero: "Rulof van der Merwe / Full-stack developer. Cape Town → Amsterdam."
- Tagline (typed-out cycling effect optional): Angular · TypeScript · Node.js · GraphQL · Rust (learning)
- One employer listed: JOBJACK, L2 Software Developer, ~3 years
- GitHub: github.com/rulofvdm

---

## `tui/` — Rust SSH TUI

### Tech Stack

- Rust stable (2021 edition)
- `russh` — async SSH server
- `ratatui` — TUI framework
- `tokio` — async runtime
- Port: 2222

### Architecture

```
tui/src/
├── main.rs       # tokio entry, russh server setup
├── server.rs     # SSH server handler (russh traits)
├── app.rs        # TUI app state
├── ui.rs         # ratatui render logic
└── content.rs    # all copy as Rust constants
```

### UX

Tab-based navigation (← →, or h/l vim-style). `q` to quit. Sections mirror the web app content.

### Deployment

Hetzner CX22 VPS. Rust binary as `systemd` service. Cross-compile for `x86_64-unknown-linux-gnu` or compile on VPS.

---

## Deployment

### Web (Cloudflare Pages)
- Build command: `cd web && ng build --configuration production`
- Output dir: `web/dist/rulof/browser`
- Custom domain: `rulof.dev`

### TUI (Hetzner)
- Run as `systemd` service (see spec for unit file)
- `systemctl enable --now rulof-tui`
