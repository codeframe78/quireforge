# GitHub Pages Plan

Status: validated architecture only. GitHub Pages is not enabled and no site has
been deployed.

## Production target

- Repository: `codeframe78/codex-linux-workbench`.
- Initial URL: `https://codeframe78.github.io/codex-linux-workbench/`.
- Source: `apps/website/` in the monorepo.
- Generator: Astro with TypeScript and static output.
- Host: GitHub Pages only.

## Official requirements verified

GitHub supports a custom Actions workflow for non-Jekyll static generators. The
documented flow checks out source, builds, uploads the static artifact with
`actions/upload-pages-artifact`, and deploys it using `actions/deploy-pages` from
the default branch. Pull-request builds should validate without deploying.

Sources:

- [GitHub publishing-source documentation](https://docs.github.com/en/pages/getting-started-with-github-pages/configuring-a-publishing-source-for-your-github-pages-site).
- [GitHub custom Pages workflows](https://docs.github.com/en/enterprise-cloud@latest/pages/getting-started-with-github-pages/using-custom-workflows-with-github-pages).
- [Astro GitHub Pages deployment guide](https://docs.astro.build/en/guides/deploy/github/).

Repository administrators must later select GitHub Actions as the Pages source.
This setting and the first deployment require explicit user approval.

## Astro path configuration

For this project site, Astro will use:

```text
site = https://codeframe78.github.io
base = /codex-linux-workbench
```

All navigation, assets, fonts, canonical URLs, social metadata, sitemap entries,
documentation links, downloads, and 404 behavior must be tested under the base
path. No root-relative path may accidentally omit the repository prefix.

The built artifact must contain the entry page at its root, not nested inside an
extra `dist` directory layer.

## Workflow design

Validation job on pull requests and default-branch pushes:

1. Check out the exact revision.
2. Set up the pinned Node version and package manager.
3. Install from the committed lockfile without lockfile changes.
4. Type-check, lint, and build the static site.
5. Validate repository-subpath links and assets.
6. Run accessibility and broken-link checks.
7. Ensure the build does not dirty the working tree.

Deployment job only on approved default-branch pushes/manual runs:

- Needs the successful validation/build job.
- Uses `pages: write` and `id-token: write`; other permissions remain read-only
  or absent.
- References the protected `github-pages` environment.
- Uses concurrency to prevent overlapping deployments without canceling an
  already-running successful deployment unsafely.
- Deploys only the exact validated uploaded artifact.

Untrusted pull requests receive no deploy job, Pages write permission, or
repository/environment secrets.

## Failed-build behavior

A build or validation failure stops before artifact deployment. Because
deployment consumes only a successful artifact, a failed build cannot replace
the currently working site.

## Custom domains

No custom domain is planned or authorized. If one is provided later, Pages
remains the host. GitHub requires configuration through repository settings or
the supported API; a `CNAME` file alone does not configure the domain for an
Actions-based Pages site. DNS changes remain manual and require explicit
approval.

See [GitHub custom-domain documentation](https://docs.github.com/en/pages/configuring-a-custom-domain-for-your-github-pages-site/managing-a-custom-domain-for-your-github-pages-site).

## Content and privacy boundary

The static site may publish release metadata, curated screenshots, documented
compatibility, roadmap, security/privacy guidance, and public source links. It
must never ingest local SQLite data, Codex session data, installed integration
lists, account/workspace details, tokens, local paths, or unredacted diagnostic
fixtures.

## Activation checklist

- [ ] User explicitly approves Pages setting change.
- [ ] Default branch and deployment environment are protected.
- [ ] Workflow action references and permissions are reviewed.
- [ ] Astro `site` and `base` match the confirmed repository.
- [ ] Local/subpath build, links, 404, sitemap, canonical URLs, and assets pass.
- [ ] Accessibility and Lighthouse targets are measured.
- [ ] No placeholder claims, fake metrics, or private data are present.
- [ ] User explicitly approves first production deployment.
