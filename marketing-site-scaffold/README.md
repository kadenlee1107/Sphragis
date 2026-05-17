# Sphragis Marketing Site — Hugo Scaffold

**SP-DOC-009 SCAFFOLD (2026-05-16).** Hugo project structure + placeholder content. Founder fills in branding decisions + deploys to Cloudflare Pages.

Quick-start:

```bash
# 1. Init Hugo + theme
cd marketing-site-scaffold/
hugo new site . --force
git submodule add https://github.com/adityatelange/hugo-PaperMod themes/PaperMod

# 2. Edit content/_index.md + content/differentiators/_index.md per the templates
# 3. Edit config.toml with your domain + author

# 4. Local preview
hugo server -D

# 5. Deploy to Cloudflare Pages
#    Dashboard: pages.cloudflare.com → Create application → Connect to Git
#    Framework: Hugo / Build cmd: hugo --minify / Output: public
```

## What's in this scaffold

| Path | Purpose |
|---|---|
| `README.md` | This file — overview + deployment instructions |
| `config.toml.template` | Hugo config with placeholders for site URL + domain + author |
| `content/_index.md.template` | Home page content with `{COMPANY_NAME}` placeholders |
| `content/differentiators/_index.md.template` | Long-form 5-differentiator page |
| `content/technical/_index.md.template` | Technical overview cross-references in-repo design docs |
| `content/contact.md.template` | Contact page with security-disclosure email placeholder |
| `content/blog/launch.md.template` | Launch blog post template |
| `layouts/.gitkeep` | Theme-override directory (mostly empty; theme provides defaults) |
| `static/.gitkeep` | Static assets (favicon, capability statement PDF goes here once rendered) |

All `.template` files have `{PLACEHOLDER}` strings the founder replaces.

## Content design (matches the strategic frame from the master plan)

Home page leads with:
> Sphragis — **a sovereign-grade attested-cave OS for the post-quantum, capability-hardware era**

Five differentiators in this order (matches REQ-STRAT-002 discipline):
1. Rust microkernel + Verus info-flow proofs
2. CNSA 2.0 PQC-only crypto
3. Attestation as kernel primitive
4. SLSA L4 reproducible builds
5. CHERI-ready architecture

Anti-features link to `ANTI_FEATURES.md` (canonical source).

## What's INTENTIONALLY NOT in this scaffold

- **Brand identity choices**: founder decides on the logo, color palette, typography. These are not technical decisions; the scaffold deliberately stays content-neutral.
- **Specific call-to-action wording**: depends on founder's go-to-market posture (consulting vs license sales vs pilots).
- **Pricing**: not appropriate for a public marketing site at this stage; surfaces via direct sales-conversation.
- **Customer case studies**: none yet; will populate as customers land.

## Hugo theme rationale

PaperMod is the recommended theme:
- Clean, fast, dev-blog-friendly
- Markdown-native; no CMS overhead
- Apache-2.0 compatible license
- Active maintenance

Alternatives (if founder prefers): Doks (docs-heavier), Hello-Friend (minimal), Anatole (academic).

## Deploying to Cloudflare Pages

1. Push the rendered Hugo project to a public GitHub repo (e.g., `kadenlee1107/sphragis-web`).
2. Cloudflare Dashboard → Workers & Pages → Create application → Pages → Connect to Git → Select repo.
3. Build settings:
   - Framework preset: Hugo
   - Build command: `hugo --minify`
   - Build output directory: `public`
4. Custom domain: `sphragis.com` (after DNS migration).

## Maintenance cadence

- **Quarterly**: refresh blog with major milestones (SBIR awards, certs achieved, partners landed)
- **Per release**: update version banner on home page
- **As-needed**: update differentiator counts if claims change (e.g., "5 differentiators" might become "6 differentiators" if a major new capability lands)

## REQ traceability

Closes REQ-DOC-009 (scaffold portion). Founder owns the brand-identity + deploy + maintenance.
