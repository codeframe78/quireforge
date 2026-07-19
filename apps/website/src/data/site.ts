export const site = {
  name: "QuireForge",
  tagline: "Build boldly. Work locally.",
  description: "An unofficial native Linux workspace for Codex.",
  origin: "https://quireforge.jamesjennison.net",
  repository: "https://github.com/codeframe78/quireforge",
} as const;

export type NavigationItem = {
  label: string;
  href: string;
};

export const primaryNavigation: NavigationItem[] = [
  { label: "Features", href: "/features/" },
  { label: "Integrations", href: "/integrations/" },
  { label: "Docs", href: "/documentation/" },
  { label: "Roadmap", href: "/roadmap/" },
];

export const footerNavigation: Array<{
  title: string;
  links: NavigationItem[];
}> = [
  {
    title: "Explore",
    links: [
      { label: "Features", href: "/features/" },
      { label: "Integrations", href: "/integrations/" },
      { label: "Compatibility", href: "/compatibility/" },
      { label: "Roadmap", href: "/roadmap/" },
    ],
  },
  {
    title: "Get QuireForge",
    links: [
      { label: "Downloads", href: "/downloads/" },
      { label: "Installation", href: "/installation/" },
      { label: "Releases", href: "/releases/" },
      { label: "GitHub", href: site.repository },
    ],
  },
  {
    title: "Project",
    links: [
      { label: "Documentation", href: "/documentation/" },
      { label: "Security & privacy", href: "/security/" },
      { label: "Contributing", href: "/contributing/" },
      { label: "About", href: "/about/" },
    ],
  },
  {
    title: "Help",
    links: [
      { label: "FAQ", href: "/faq/" },
      { label: "Troubleshooting", href: "/troubleshooting/" },
      { label: "Report an issue", href: `${site.repository}/issues` },
      {
        label: "Support policy",
        href: `${site.repository}/blob/main/SUPPORT.md`,
      },
    ],
  },
];

export type ContentSection = {
  heading: string;
  paragraphs?: string[];
  items?: string[];
};

export type SitePage = {
  slug: string;
  eyebrow: string;
  title: string;
  description: string;
  status?: string;
  sections: ContentSection[];
  links?: NavigationItem[];
};

export const sitePages: SitePage[] = [
  {
    slug: "features",
    eyebrow: "Product direction",
    title: "A Linux workspace built around the repository you already have.",
    description:
      "QuireForge is being designed for direct local-directory work, observable Codex tasks, deliberate approvals, and native Linux workflows.",
    status: "Design validated · implementation pending",
    sections: [
      {
        heading: "Projects stay where they are",
        paragraphs: [
          "Attach an existing local directory and keep working against the original path. QuireForge will not require an import folder, background upload, or duplicate repository.",
        ],
        items: [
          "Persistent project-to-directory associations",
          "Git repository, worktree, branch, and dirty-state detection",
          "Missing, moved, read-only, removable, and network-path states",
          "Explicit detach and re-link actions that never delete source files",
        ],
      },
      {
        heading: "Codex work you can follow",
        paragraphs: [
          "The planned desktop interface separates commentary, commands, approvals, file changes, plans, and final results instead of flattening them into terminal text.",
        ],
        items: [
          "Concurrent conversations and isolated worktrees",
          "Streamed progress with interruption and recovery",
          "Exact command and filesystem approval details",
          "Git status, diff review, previews, and a real PTY terminal",
        ],
      },
      {
        heading: "Capability-aware by design",
        paragraphs: [
          "QuireForge will probe the installed Codex version and expose only supported models, reasoning levels, session operations, and integrations. Unknown support is reported as unknown—not promoted as compatibility.",
        ],
      },
    ],
    links: [
      {
        label: "Read the architecture",
        href: `${site.repository}/blob/main/docs/ARCHITECTURE.md`,
      },
      { label: "Follow the roadmap", href: "/roadmap/" },
    ],
  },
  {
    slug: "integrations",
    eyebrow: "Integration Center",
    title: "Supported integrations, without an invented catalog.",
    description:
      "QuireForge will surface apps and connectors, plugins, skills, MCP servers, and marketplaces only when supported Codex interfaces expose them.",
    status: "Compatibility research complete · UI pending",
    sections: [
      {
        heading: "Five categories, kept distinct",
        items: [
          "Apps and connectors link Codex to supported external services and may require official authorization.",
          "Plugins are installable Codex bundles that may include skills, connectors, MCP configuration, hooks, and assets.",
          "Skills are reusable workflows that may be built in, local, or plugin-provided.",
          "MCP servers expose local or remote tools and may require separate authentication.",
          "Marketplaces are curated, managed, repository, community, or local catalog sources.",
        ],
      },
      {
        heading: "Compatibility is contextual",
        paragraphs: [
          "Availability can depend on the installed Codex version, account plan, workspace policy, administrator approval, region, operating system, runtime dependencies, network access, and authentication state.",
          "When Codex cannot enumerate or manage a class of integrations programmatically, QuireForge will explain the limitation and offer the closest official workflow.",
        ],
      },
      {
        heading: "Installation is a security decision",
        paragraphs: [
          "Plugins, hooks, MCP servers, and marketplace sources can execute code or access data. QuireForge will show publisher and source metadata, bundled capabilities, requested filesystem and network access, and any unknowns before confirmation.",
          "Connector credentials remain owned by Codex, the connector, or the operating system. QuireForge will never request passwords or store OAuth tokens in its SQLite database.",
        ],
      },
    ],
    links: [
      {
        label: "Integration research",
        href: `${site.repository}/blob/main/docs/CODEX-INTEGRATION.md`,
      },
      {
        label: "Integration security",
        href: `${site.repository}/blob/main/docs/THREAT-MODEL.md`,
      },
    ],
  },
  {
    slug: "downloads",
    eyebrow: "Downloads",
    title: "Packages will arrive after the application earns them.",
    description:
      "QuireForge does not have an installable release yet. AppImage and Debian packages are scheduled for the packaging milestone after desktop functionality and security testing.",
    status: "No downloads available",
    sections: [
      {
        heading: "Planned formats",
        items: [
          "AppImage for a portable Linux installation",
          "Debian package for supported Ubuntu systems",
          "Checksums and provenance attached to approved GitHub Releases",
          "Documented upgrade and uninstall behavior that preserves projects and Codex sessions",
        ],
      },
      {
        heading: "Avoid unofficial artifacts",
        paragraphs: [
          "Until the project publishes an approved release, files claiming to be QuireForge packages are not project releases. Future downloads will link directly to the dedicated GitHub repository and include verification instructions.",
        ],
      },
    ],
    links: [
      { label: "Watch GitHub releases", href: `${site.repository}/releases` },
      {
        label: "Packaging plan",
        href: `${site.repository}/blob/main/docs/ROADMAP.md`,
      },
    ],
  },
  {
    slug: "installation",
    eyebrow: "Installation",
    title: "Installation guidance will follow verified packages.",
    description:
      "There is no supported QuireForge installation today. Website and desktop development are documented, but the verified scaffold is not a package or release.",
    status: "Not yet installable",
    sections: [
      {
        heading: "Target environment",
        items: [
          "Ubuntu LTS is the primary distribution target.",
          "Wayland and X11 behavior will be tested separately.",
          "Codex and Git remain external runtime prerequisites.",
          "The application will use XDG configuration, data, cache, and state locations.",
        ],
      },
      {
        heading: "Uninstall safety",
        paragraphs: [
          "Removing QuireForge must not delete attached directories, Git repositories, worktrees, uncommitted changes, or Codex-owned sessions. Application metadata cleanup will remain a separate explicit operation.",
        ],
      },
    ],
    links: [{ label: "Track packaging progress", href: "/roadmap/" }],
  },
  {
    slug: "documentation",
    eyebrow: "Documentation",
    title: "The decisions are public before the implementation lands.",
    description:
      "Architecture, compatibility findings, security boundaries, hosting decisions, and milestone status are maintained in the repository.",
    status: "Discovery documentation available",
    sections: [
      {
        heading: "Start here",
        items: [
          "Architecture and service boundaries",
          "Codex integration research and feature parity",
          "Compatibility and Linux prerequisites",
          "Threat model and directory-attachment safety",
          "Cloudflare Pages capability and deployment planning",
        ],
      },
      {
        heading: "Documentation grows with the product",
        paragraphs: [
          "Website build and test guides are executable today. Desktop, packaging, user, integration, and release guides will become executable as their owning milestones are implemented. Planned behavior is always labeled separately from working behavior.",
        ],
      },
    ],
    links: [
      {
        label: "Browse repository docs",
        href: `${site.repository}/tree/main/docs`,
      },
      { label: "Read the README", href: `${site.repository}#readme` },
    ],
  },
  {
    slug: "compatibility",
    eyebrow: "Compatibility",
    title: "Detected at runtime, stated with evidence.",
    description:
      "QuireForge will distinguish stable official interfaces, experimental official interfaces, local functionality, and unsupported features.",
    status: "Ubuntu and Codex validation in progress",
    sections: [
      {
        heading: "Current baseline",
        items: [
          "Ubuntu LTS is the primary Linux target; supported versions are not yet declared.",
          "Codex capability findings currently reflect CLI 0.144.6 and must be probed at runtime.",
          "Cloudflare Pages is compatible with the static Astro website design.",
          "AppImage and Debian packaging remain unimplemented and unverified.",
        ],
      },
      {
        heading: "Honest degradation",
        paragraphs: [
          "Unavailable controls will be disabled with useful explanations. QuireForge will not infer connector access, model availability, plugin support, or administrator permission from documentation alone.",
        ],
      },
    ],
    links: [
      {
        label: "Detailed compatibility matrix",
        href: `${site.repository}/blob/main/docs/COMPATIBILITY.md`,
      },
    ],
  },
  {
    slug: "roadmap",
    eyebrow: "Public roadmap",
    title: "Twenty-one gated milestones, one reviewable step at a time.",
    description:
      "QuireForge is built milestone by milestone, with model selection, acceptance criteria, tests, documentation, review, and explicit approval for external actions.",
    status: "Milestone 5 complete locally · Milestone 6 next",
    sections: [
      {
        heading: "Completed locally",
        items: [
          "Milestone 0: project, Codex, GitHub, hosting, DNS, and feasibility discovery",
          "Milestone 1: permanent identity reconciliation and open-source governance",
          "Milestone 2: brand consumption, Astro foundation, responsive layout, accessibility, and Cloudflare-compatible static output",
          "Milestone 3: Tauri, React, TypeScript, and Rust desktop scaffold with typed IPC and local Wayland verification",
          "Milestone 4: supervised Codex app-server probe, normalized runtime contracts, deterministic mocks, and bounded failure recovery",
          "Milestone 5: Codex-owned browser/device authentication, normalized account state, cancellation, logout confirmation, and redacted recovery",
        ],
      },
      {
        heading: "Next gated milestone",
        items: ["Milestone 6: direct local-directory attachment"],
      },
      {
        heading: "Later",
        items: [
          "Milestone 7: conversation MVP in the verified attached directory",
          "Milestones 13–14: integration discovery and management",
          "Milestones 19–20: packages, approved deployment, and beta release",
        ],
      },
    ],
    links: [
      {
        label: "Full roadmap and acceptance model",
        href: `${site.repository}/blob/main/docs/ROADMAP.md`,
      },
    ],
  },
  {
    slug: "releases",
    eyebrow: "Releases",
    title: "No release until the core workflow is safe and testable.",
    description:
      "The repository has not published a QuireForge release. Approved packages will be distributed through GitHub Releases with checksums and release notes.",
    status: "Pre-release development",
    sections: [
      {
        heading: "Release requirements",
        items: [
          "Install, upgrade, and uninstall tests on supported Ubuntu",
          "Verified local-directory and Codex working-directory behavior",
          "Security, accessibility, and integration supply-chain review",
          "Reproducible AppImage and Debian artifacts with checksums",
          "Separately approved publication and website download links",
        ],
      },
    ],
    links: [
      {
        label: "View changelog",
        href: `${site.repository}/blob/main/CHANGELOG.md`,
      },
      { label: "GitHub Releases", href: `${site.repository}/releases` },
    ],
  },
  {
    slug: "security",
    eyebrow: "Security & privacy",
    title: "Local access deserves explicit boundaries.",
    description:
      "QuireForge is designed to keep directory access, command approval, integration permissions, and credential ownership visible and separate.",
    status: "Threat model published · implementation pending",
    sections: [
      {
        heading: "Data ownership",
        items: [
          "QuireForge owns only its application metadata.",
          "Git remains authoritative for repository state.",
          "Codex remains authoritative for authentication, sessions, and supported integration state.",
          "Connector secrets never belong in QuireForge SQLite or support bundles.",
        ],
      },
      {
        heading: "Report privately",
        paragraphs: [
          "Do not post credentials, private source code, connector data, or exploit details in public issues. Follow the repository security policy to request a private reporting route.",
        ],
      },
    ],
    links: [
      {
        label: "Security policy",
        href: `${site.repository}/blob/main/SECURITY.md`,
      },
      {
        label: "Threat model",
        href: `${site.repository}/blob/main/docs/THREAT-MODEL.md`,
      },
    ],
  },
  {
    slug: "contributing",
    eyebrow: "Contributing",
    title: "Help shape a careful Linux-native Codex workspace.",
    description:
      "Contributions are welcome across Rust, TypeScript, Linux integration, accessibility, security, testing, documentation, and visual design.",
    status: "Governance baseline available",
    sections: [
      {
        heading: "Before opening a pull request",
        items: [
          "Read the active milestone and relevant architecture decisions.",
          "Keep changes focused and preserve existing work.",
          "Add deterministic tests without billable model calls or real authorization.",
          "Document security, privacy, accessibility, and compatibility effects.",
          "Never submit credentials, personal Codex data, or private fixtures.",
        ],
      },
      {
        heading: "License and conduct",
        paragraphs: [
          "QuireForge uses the Apache License 2.0. Community participation follows the Contributor Covenant-based Code of Conduct in the repository.",
        ],
      },
    ],
    links: [
      {
        label: "Contribution guide",
        href: `${site.repository}/blob/main/CONTRIBUTING.md`,
      },
      { label: "Open issues", href: `${site.repository}/issues` },
    ],
  },
  {
    slug: "faq",
    eyebrow: "FAQ",
    title: "Straight answers for an early-stage project.",
    description:
      "QuireForge is public by design about what exists, what is planned, and what depends on supported Codex interfaces.",
    sections: [
      {
        heading: "Is QuireForge made by OpenAI?",
        paragraphs: [
          "No. QuireForge is an independent, unofficial community project. OpenAI does not make, endorse, support, or distribute it.",
        ],
      },
      {
        heading: "Can I install it today?",
        paragraphs: [
          "Not yet. The website, desktop scaffold, and non-billable Codex runtime probe are locally verified, but neither an AppImage nor a Debian package has been produced. The downloads page will remain explicit until verified packages exist.",
        ],
      },
      {
        heading: "Will it copy or upload my repository?",
        paragraphs: [
          "The product requirement is to attach and work against your original local directory in place. QuireForge must not silently copy, relocate, upload, or replace it.",
        ],
      },
      {
        heading: "Will every ChatGPT app work?",
        paragraphs: [
          "No such claim is possible. QuireForge will show only integrations exposed through supported Codex or ChatGPT mechanisms for the installed version, account, region, and workspace policy.",
        ],
      },
    ],
    links: [
      { label: "Read troubleshooting guidance", href: "/troubleshooting/" },
    ],
  },
  {
    slug: "troubleshooting",
    eyebrow: "Troubleshooting",
    title: "Start with the boundary that owns the failure.",
    description:
      "Future diagnostics will distinguish QuireForge, Codex, Git, the project directory, Linux desktop services, and integrations instead of collapsing errors together.",
    status: "End-user diagnostics not implemented",
    sections: [
      {
        heading: "Before reporting a development issue",
        items: [
          "Record the QuireForge revision and Linux distribution.",
          "Record Codex and Git versions without including credentials.",
          "Describe whether the path is local, removable, network-backed, a symlink, or a worktree.",
          "Sanitize usernames, absolute paths, source code, account IDs, private URLs, and tokens.",
        ],
      },
      {
        heading: "Account and policy limitations",
        paragraphs: [
          "QuireForge cannot grant Codex entitlements, workspace permissions, connector authorization, or regional availability. Those failures must be handled through the official provider or workspace administrator.",
        ],
      },
    ],
    links: [
      {
        label: "Support guide",
        href: `${site.repository}/blob/main/SUPPORT.md`,
      },
      {
        label: "Report a sanitized issue",
        href: `${site.repository}/issues/new/choose`,
      },
    ],
  },
  {
    slug: "about",
    eyebrow: "About",
    title: "A native Linux home for deliberate Codex work.",
    description:
      "QuireForge exists to make local project identity, streamed work, approvals, Git review, terminals, and supported integrations feel coherent on Linux.",
    sections: [
      {
        heading: "Why QuireForge",
        paragraphs: [
          "A quire is a gathered set of pages; a forge is where tools are shaped through deliberate work. The name reflects a workspace that brings project context together without taking ownership of the files themselves.",
        ],
      },
      {
        heading: "Independent by design",
        paragraphs: [
          "The visual identity and implementation are original. QuireForge does not redistribute or reverse engineer OpenAI's Windows application, imitate protected branding, or promise proprietary access without a documented interface.",
        ],
      },
      {
        heading: "Open source",
        paragraphs: [
          "The project is developed publicly under the Apache License 2.0 with a documented roadmap, security boundaries, and milestone acceptance process.",
        ],
      },
    ],
    links: [
      { label: "View the source", href: site.repository },
      { label: "Read the roadmap", href: "/roadmap/" },
    ],
  },
];

export const requiredPageSlugs = [
  "features",
  "integrations",
  "downloads",
  "installation",
  "documentation",
  "compatibility",
  "roadmap",
  "releases",
  "security",
  "contributing",
  "faq",
  "troubleshooting",
  "about",
] as const;
