# Web (Angular)

Static personal site for rulof.dev. Deployed to Cloudflare Pages.

## Dev

From repo root, optional Nix shell: `nix develop` (Node 22 + Angular CLI).

```bash
cd web
npm install
ng serve
```

Open http://localhost:4200/

## Production build

```bash
ng build --configuration production
```

Output: `dist/rulof/browser/`

## Tests & lint

```bash
ng test
ng lint
```
