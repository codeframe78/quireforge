import {
  downloadAvailability,
  getPublishedRelease,
  validateDownloadAvailability,
} from "./downloads";

export const site = {
  name: "QuireForge",
  tagline: "Build boldly. Work locally.",
  description:
    "An early-stage native Linux workspace concept for local, observable, and approval-aware Codex workflows.",
  origin: "https://quireforge.jamesjennison.net",
  masterOrigin: "https://jamesjennison.net",
  statusOrigin: "https://status.jamesjennison.net",
  repository: "https://github.com/James-Jennison/quireforge",
  securityReportUrl: "https://github.com/codeframe78",
} as const;

const availabilityErrors = validateDownloadAvailability(
  downloadAvailability,
  site.origin,
);
if (availabilityErrors.length > 0) {
  throw new Error(
    `Invalid download availability: ${availabilityErrors.join("; ")}`,
  );
}
export const publishedRelease = getPublishedRelease(downloadAvailability);
if (publishedRelease && !site.securityReportUrl) {
  throw new Error(
    "A published release requires an approved private security-reporting URL",
  );
}

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
    title: "Availability",
    links: [
      { label: "Downloads", href: "/downloads/" },
      { label: "Installation", href: "/installation/" },
      { label: "Releases", href: "/releases/" },
      { label: "Project status", href: "/roadmap/" },
    ],
  },
  {
    title: "Project",
    links: [
      { label: "Documentation", href: "/documentation/" },
      { label: "Security & privacy", href: "/security/" },
      { label: "Development", href: "/contributing/" },
      { label: "Source on GitHub", href: site.repository },
      { label: "About", href: "/about/" },
    ],
  },
  {
    title: "Ecosystem",
    links: [
      { label: "James Jennison", href: site.masterOrigin },
      { label: "All projects", href: `${site.masterOrigin}/projects/` },
      { label: "Service status", href: site.statusOrigin },
      { label: "Contact", href: `${site.masterOrigin}/contact/` },
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
    title: "A Linux workspace shaped around the project you already have.",
    description:
      "QuireForge is being designed for direct local-directory work, observable Codex tasks, deliberate approvals, and native Linux workflows.",
    status: "Product direction · public source",
    sections: [
      {
        heading: "Projects stay where they are",
        paragraphs: [
          "The product direction is to attach an existing local directory and keep working against its original location. QuireForge is not intended to require an import folder, background upload, or duplicate project tree.",
        ],
        items: [
          "Persistent project-to-directory associations",
          "Visible Git and worktree context",
          "Clear missing, moved, read-only, removable, and network-path states",
          "Explicit detach and re-link actions that do not delete project files",
        ],
      },
      {
        heading: "Codex work you can follow",
        paragraphs: [
          "The planned desktop experience separates commentary, commands, approvals, file changes, plans, and final results instead of flattening every event into an opaque transcript.",
        ],
        items: [
          "Observable progress and interruption states",
          "Clear command and filesystem approval details",
          "Git-aware review and project context",
          "Native Linux handoffs and terminal workflows where appropriate",
        ],
      },
      {
        heading: "Capability-aware by design",
        paragraphs: [
          "QuireForge is intended to show only capabilities supported by the installed Codex environment. Unknown support should remain unknown rather than being promoted as compatibility.",
        ],
      },
    ],
    links: [
      { label: "Follow the public roadmap", href: "/roadmap/" },
      { label: "Read the project guide", href: "/documentation/" },
    ],
  },
  {
    slug: "integrations",
    eyebrow: "Integration Center",
    title: "Supported integrations, without an invented catalog.",
    description:
      "QuireForge is intended to surface apps and connectors, plugins, skills, MCP servers, and marketplaces only when supported Codex interfaces expose them.",
    status: "Product direction · availability not promised",
    sections: [
      {
        heading: "Five categories, kept distinct",
        items: [
          "Apps and connectors link Codex to supported external services and may require official authorization.",
          "Plugins are installable bundles that may include skills, connectors, MCP configuration, hooks, or assets.",
          "Skills are reusable workflows that may be built in, local, or plugin-provided.",
          "MCP servers expose local or remote tools and may require separate authentication.",
          "Marketplaces are catalog sources with their own publishers, trust boundaries, and policies.",
        ],
      },
      {
        heading: "Compatibility is contextual",
        paragraphs: [
          "Availability may depend on the installed Codex version, account plan, workspace policy, administrator approval, region, operating system, runtime dependencies, network access, and authentication state.",
          "When an integration cannot be inspected or managed through a supported interface, QuireForge should explain that boundary rather than fabricate control.",
        ],
      },
      {
        heading: "Installation is a security decision",
        paragraphs: [
          "Plugins, hooks, MCP servers, and marketplace sources can execute code or access data. The intended review experience makes available publisher, source, capability, permission, and uncertainty information visible before confirmation.",
          "Connector credentials remain owned by Codex, the connector, or the operating system. QuireForge is not intended to collect service passwords or copy provider-owned authorization tokens into its application data.",
        ],
      },
    ],
    links: [
      { label: "Review the product principles", href: "/features/" },
      { label: "Read security & privacy", href: "/security/" },
    ],
  },
  {
    slug: "downloads",
    eyebrow: "Downloads",
    title: publishedRelease
      ? `Download QuireForge ${publishedRelease.version}.`
      : "Packages will arrive after the application earns them.",
    description: publishedRelease
      ? "Choose the approved x86_64 package for your Ubuntu system, then verify its published SHA-256 checksum before running it."
      : "QuireForge has no public or supported download. AppImage and Debian packages remain planned for a later, separately approved release milestone.",
    status: downloadAvailability.statusLabel,
    sections: publishedRelease
      ? [
          {
            heading: "Approved packages",
            items: publishedRelease.downloads.map(
              (download) =>
                `${download.filename} · ${download.byteSize.toLocaleString("en-US")} bytes · SHA-256 ${download.sha256}`,
            ),
          },
          {
            heading: "Verify before opening",
            paragraphs: [
              "Download SHA256SUMS and your selected package into the same directory. Run sha256sum --check --ignore-missing SHA256SUMS and continue only when the selected filename reports OK.",
              "QuireForge is an unofficial community project. OpenAI does not make, endorse, support, or distribute these packages.",
            ],
          },
        ]
      : [
          {
            heading: "Planned formats",
            items: [
              "AppImage for a portable Linux installation",
              "Debian package for supported Ubuntu systems",
              "Published checksums and release notes for every approved artifact",
              "Documented upgrade and uninstall behavior that preserves user projects",
            ],
          },
          {
            heading: "Avoid unofficial artifacts",
            paragraphs: [
              "No file currently claiming to be a QuireForge installer is an approved project release. When verified packages exist, this website will provide the authoritative download and verification guidance.",
            ],
          },
        ],
    links: publishedRelease
      ? [
          ...publishedRelease.downloads.map((download) => ({
            label:
              download.format === "appimage"
                ? "Download AppImage"
                : "Download Debian package",
            href: download.url,
          })),
          { label: "Download SHA256SUMS", href: publishedRelease.checksumUrl },
          {
            label: "Review release manifest",
            href: publishedRelease.manifestUrl,
          },
          { label: "Read installation guidance", href: "/installation/" },
        ]
      : [
          { label: "Read the release policy", href: "/releases/" },
          { label: "Follow the public roadmap", href: "/roadmap/" },
        ],
  },
  {
    slug: "installation",
    eyebrow: "Installation",
    title: publishedRelease
      ? `Install QuireForge ${publishedRelease.version}.`
      : "Installation guidance will follow verified packages.",
    description: publishedRelease
      ? "The beta targets x86_64 Ubuntu 22.04 or newer on GNOME Wayland or X11, with compatible Codex and Git installations available on the system."
      : "There is no supported QuireForge installation today. Development builds are not public packages or releases.",
    status: publishedRelease
      ? `Beta ${publishedRelease.version} · x86_64 Ubuntu`
      : "Not yet installable",
    sections: publishedRelease
      ? [
          {
            heading: "Before installation",
            items: [
              "Confirm the package and SHA256SUMS came from this website.",
              "Verify the selected package and require an OK result before continuing.",
              "Install compatible Codex and Git separately; QuireForge does not bundle or own their credentials or data.",
            ],
          },
          {
            heading: "AppImage",
            items: [
              "Mark the downloaded AppImage executable with chmod 0755, then launch it directly.",
              "If FUSE is unavailable, use the AppImage runtime's --appimage-extract-and-run mode.",
              "The AppImage is not registered with a distribution package manager and must be replaced manually for upgrades.",
            ],
          },
          {
            heading: "Debian package",
            items: [
              "Install the local file with apt so Ubuntu can resolve the declared WebKitGTK and GTK dependencies.",
              "Remove it with apt remove quireforge. Package removal does not delete attached projects, Git repositories, Codex state, or QuireForge user metadata.",
            ],
          },
        ]
      : [
          {
            heading: "Target environment",
            items: [
              "Ubuntu LTS is the initial Linux distribution target.",
              "Wayland and X11 behavior require separate validation.",
              "Codex and Git are expected to remain external prerequisites.",
              "Application data should follow Linux XDG location conventions.",
            ],
          },
          {
            heading: "Uninstall safety",
            paragraphs: [
              "Removing QuireForge must not delete attached directories, Git repositories, worktrees, uncommitted changes, or Codex-owned sessions. Application metadata cleanup should remain a separate, explicit operation.",
            ],
          },
        ],
    links: [
      { label: "Check download availability", href: "/downloads/" },
      { label: "Track the release path", href: "/roadmap/" },
    ],
  },
  {
    slug: "documentation",
    eyebrow: "Documentation",
    title: "Public guidance backed by inspectable source.",
    description:
      "This website documents QuireForge's public purpose, product direction, availability, compatibility boundaries, and safety principles.",
    status: "Public project overview available",
    sections: [
      {
        heading: "Available here",
        items: [
          "Product direction and local-project principles",
          "Integration categories and capability boundaries",
          "Compatibility targets and current availability",
          "Security, privacy, release, and troubleshooting guidance",
          "A deliberately high-level public roadmap",
        ],
      },
      {
        heading: "Public source and safety boundary",
        paragraphs: [
          "Source code, architecture decisions, milestones, development activity, and issue tracking are public in the QuireForge GitHub repository.",
          "Credentials, local project content, Codex account data, provider identifiers, and private diagnostics remain outside the repository.",
          "Planned behavior is labeled separately from available behavior throughout this site.",
        ],
      },
    ],
    links: [
      { label: "Explore the features", href: "/features/" },
      { label: "Read the FAQ", href: "/faq/" },
    ],
  },
  {
    slug: "compatibility",
    eyebrow: "Compatibility",
    title: "Detected at runtime, stated with evidence.",
    description:
      "QuireForge is intended to distinguish supported interfaces, experimental interfaces, local functionality, and unavailable features.",
    status: publishedRelease
      ? `Beta ${publishedRelease.version} · x86_64 Ubuntu 22.04+`
      : "Linux support targets under evaluation",
    sections: [
      {
        heading: "Current public baseline",
        items: publishedRelease
          ? [
              "The initial beta target is x86_64 Ubuntu 22.04 or newer on GNOME Wayland and X11.",
              "AppImage and Debian packages are available only through the approved Downloads page.",
              "Codex and Git are external prerequisites; Codex capability is checked at runtime.",
              "Arm64, non-Ubuntu distributions, and non-GNOME desktops are not part of the initial support statement.",
              "The project website and approved packages are served from the owner-hosted QuireForge origin behind Cloudflare DNS and proxying.",
            ]
          : [
              "Ubuntu LTS is the initial target; supported versions are not yet declared.",
              "Codex capabilities may vary by installed version and should be checked at runtime.",
              "Wayland and X11 behavior require separate evidence.",
              "AppImage and Debian packaging remain unavailable.",
              "The static project website is public from a Webuzo-managed origin behind Cloudflare DNS and proxying.",
            ],
      },
      {
        heading: "Honest degradation",
        paragraphs: [
          "Unavailable controls should remain disabled with useful explanations. QuireForge should not infer connector access, model availability, plugin support, or administrator permission from documentation alone.",
        ],
      },
    ],
    links: [
      { label: "Read installation targets", href: "/installation/" },
      { label: "Understand integrations", href: "/integrations/" },
    ],
  },
  {
    slug: "roadmap",
    eyebrow: "Public roadmap",
    title: "A gated path from product foundation to verified release.",
    description:
      "QuireForge moves through reviewable phases while detailed milestones, source activity, and internal implementation records remain private.",
    status: publishedRelease
      ? `Public beta ${publishedRelease.version}`
      : "Early development · no release date announced",
    sections: [
      {
        heading: "Foundation",
        items: [
          "Original project identity and product principles established",
          "Static website and public information architecture deployed and validated",
          "Security, privacy, accessibility, and compatibility treated as product boundaries",
        ],
      },
      {
        heading: "Public-source development",
        items: [
          "Native Linux workspace and local-project workflows",
          "Observable Codex tasks, approvals, review, and recovery",
          "Capability-aware integration and desktop behavior",
        ],
      },
      publishedRelease
        ? {
            heading: "Public beta",
            items: [
              "Verified AppImage and Debian packages with published checksums and manifest",
              "Initial x86_64 Ubuntu support statement and installation guidance",
              "Authenticated product home, bounded usage visibility, and established local-workspace flows",
              "Ongoing compatibility, security, accessibility, and upgrade validation before a stable release",
            ],
          }
        : {
            heading: "Before public availability",
            items: [
              "Revalidate security, accessibility, performance, and compatibility against release packages",
              "Produce and verify supported installation packages",
              "Publish release notes, checksums, installation guidance, and known limitations",
              "Obtain separate approval for beta publication and downloads",
            ],
          },
    ],
    links: [
      { label: "Check current availability", href: "/downloads/" },
      { label: "Read the project guide", href: "/documentation/" },
    ],
  },
  {
    slug: "releases",
    eyebrow: "Releases",
    title: publishedRelease
      ? `QuireForge ${publishedRelease.version} beta.`
      : "No release until the core workflow is safe and testable.",
    description: publishedRelease
      ? "The first public beta packages the native Linux workspace, authenticated Codex access gate, local-project workflows, and bounded integrations for the declared Ubuntu target."
      : "QuireForge has not published an application release. Future packages require verification, documentation, and separate owner approval.",
    status: publishedRelease
      ? `Published ${publishedRelease.publishedAt.slice(0, 10)}`
      : "Pre-release development",
    sections: publishedRelease
      ? [
          {
            heading: "Highlights",
            items: [
              "Attach existing local projects in place and preserve their filesystem ownership.",
              "Run observable Codex conversations with explicit approvals, Git review, worktrees, and native terminals.",
              "Use the Codex-owned authentication flow and see documented remaining usage when Codex provides it.",
              "Inspect bounded integrations, preview safe project files, and attach reviewed PNG/JPEG images.",
            ],
          },
          {
            heading: "Known limitations",
            items: [
              "The initial beta targets x86_64 Ubuntu 22.04 or newer on GNOME Wayland or X11.",
              "Codex and Git are external prerequisites, and integration availability remains environment-dependent.",
              "The AppImage has no automatic updater; the Debian package is not an APT-repository release.",
              "Scheduled-task discovery is read-only and several advanced operations remain deliberately unavailable.",
            ],
          },
        ]
      : [
          {
            heading: "Release requirements",
            items: [
              "Install, upgrade, and uninstall tests on declared Linux targets",
              "Verified local-project and Codex working-directory behavior",
              "Security, accessibility, and integration supply-chain review",
              "Reproducible AppImage and Debian artifacts with checksums",
              "Separately approved publication and website download links",
            ],
          },
          {
            heading: "Authoritative release channel",
            paragraphs: [
              "When a release is approved, this website will identify the supported version, provide verification guidance, and link only to the owner-approved distribution location.",
            ],
          },
        ],
    links: [
      { label: "Check downloads", href: "/downloads/" },
      { label: "Review compatibility", href: "/compatibility/" },
    ],
  },
  {
    slug: "security",
    eyebrow: "Security & privacy",
    title: "Local access deserves explicit boundaries.",
    description:
      "QuireForge is designed to keep directory access, command approval, integration permissions, and credential ownership visible and separate.",
    status: publishedRelease
      ? `Beta ${publishedRelease.version} security boundary`
      : "Public principles · private engineering review",
    sections: [
      {
        heading: "Data ownership",
        items: [
          "QuireForge should own only the application metadata it needs.",
          "Git remains authoritative for repository state.",
          "Codex remains authoritative for its authentication, sessions, and supported integration state.",
          "Connector secrets do not belong in QuireForge application data or support bundles.",
        ],
      },
      publishedRelease
        ? {
            heading: "Report a beta security issue privately",
            paragraphs: [
              "Use the security-contact link to request a private reporting channel without including vulnerability details in the initial message. Do not send credentials, private source code, access tokens, or proof-of-concept exploits through ordinary public contact channels.",
            ],
          }
        : {
            heading: "Security reporting before release",
            paragraphs: [
              "There is no public application release to report against today. A dedicated private security-reporting path and disclosure guidance must be published before beta availability.",
              "Do not send credentials, private source code, access tokens, or exploit details through ordinary public contact channels.",
            ],
          },
    ],
    links: [
      ...(publishedRelease && site.securityReportUrl
        ? [
            {
              label: "Request a private security channel",
              href: site.securityReportUrl,
            },
          ]
        : []),
      { label: "Read the project FAQ", href: "/faq/" },
      {
        label: "General project contact",
        href: `${site.masterOrigin}/contact/`,
      },
    ],
  },
  {
    slug: "contributing",
    eyebrow: "Development",
    title: "Inspect the source and propose focused changes.",
    description:
      "QuireForge source, issues, pull requests, and contribution guidance are public on GitHub.",
    status: "Public source",
    sections: [
      {
        heading: "Current boundary",
        items: [
          "Review the roadmap, architecture, and relevant decisions before proposing substantial work.",
          "Use sanitized fixtures and never submit credentials, personal Codex data, private project content, or provider diagnostics.",
          "Fork-origin pull requests cannot execute code on QuireForge's persistent self-hosted runners.",
          "A passing automated check does not replace security, architecture, accessibility, or compatibility review.",
        ],
      },
      {
        heading: "Contribution workflow",
        paragraphs: [
          "Open an issue before investing heavily in architectural, security-sensitive, integration, storage, packaging, or externally visible changes. Focus pull requests, explain their security and compatibility effects, and run the documented local checks.",
        ],
      },
    ],
    links: [
      { label: "View source on GitHub", href: site.repository },
      { label: "Browse issues", href: `${site.repository}/issues` },
      { label: "Follow the public roadmap", href: "/roadmap/" },
    ],
  },
  {
    slug: "faq",
    eyebrow: "FAQ",
    title: "Straight answers for an early-stage project.",
    description:
      "QuireForge's public website separates product direction from current availability and links to its inspectable source.",
    sections: [
      {
        heading: "Is QuireForge made by OpenAI?",
        paragraphs: [
          "No. QuireForge is an independent, unofficial project. OpenAI does not make, endorse, support, or distribute it.",
        ],
      },
      {
        heading: "Can I install it today?",
        paragraphs: [
          publishedRelease
            ? `Yes. QuireForge ${publishedRelease.version} is available as a verified x86_64 AppImage and Debian package for the declared Ubuntu beta target. Use only the Downloads page and verify SHA256SUMS before installation.`
            : "No. There is no approved AppImage, Debian package, beta, or supported installation. The downloads page will remain explicit until verified packages exist.",
        ],
      },
      {
        heading: "Is the source repository public?",
        paragraphs: [
          "Yes. The QuireForge source, issue tracking, milestones, pull requests, and development activity are public on GitHub.",
        ],
      },
      {
        heading: "Will it copy or upload my project?",
        paragraphs: [
          "The product requirement is to attach and work against an original local directory in place. QuireForge must not silently copy, relocate, upload, or replace it.",
        ],
      },
      {
        heading: "Will every ChatGPT app work?",
        paragraphs: [
          "No such claim is possible. QuireForge should show only integrations supported for the installed Codex environment, account, region, and workspace policy.",
        ],
      },
    ],
    links: [
      { label: "Read troubleshooting guidance", href: "/troubleshooting/" },
      { label: "Check current availability", href: "/downloads/" },
    ],
  },
  {
    slug: "troubleshooting",
    eyebrow: "Troubleshooting",
    title: publishedRelease
      ? `Troubleshoot QuireForge ${publishedRelease.version}.`
      : "There is no public application build to support yet.",
    description: publishedRelease
      ? "Separate package verification, Codex runtime/account state, Git/project access, Linux desktop services, and integrations before deciding what failed."
      : "Current guidance covers website availability and future diagnostic boundaries without implying that a supported QuireForge package exists.",
    status: publishedRelease
      ? "Public beta troubleshooting"
      : "Application support not yet open",
    sections: [
      publishedRelease
        ? {
            heading: "Start with the boundary that failed",
            items: [
              "Recheck the package against the published SHA256SUMS before debugging it.",
              "If the account gate remains visible, verify the compatible Codex runtime and use only the Codex-owned sign-in flow.",
              "If a project is unavailable, review its current path, mount, permissions, Git state, and project instructions without relocating it.",
              "For AppImage FUSE errors, use the documented --appimage-extract-and-run fallback.",
              "Never include credentials, tokens, private source, or personal Codex data in an ordinary support request.",
            ],
          }
        : {
            heading: "Before public release",
            items: [
              "Do not install files presented as unofficial QuireForge packages.",
              "Use the downloads page to confirm whether an approved release exists.",
              "Check the public service-status page if this website is unavailable.",
              "Never include credentials, tokens, private source, or personal Codex data in ordinary support requests.",
            ],
          },
      {
        heading: "Future diagnostic boundary",
        paragraphs: [
          "Supported diagnostics should distinguish QuireForge, Codex, Git, the selected project directory, Linux desktop services, and integrations instead of collapsing unrelated failures together.",
        ],
      },
    ],
    links: [
      { label: "Check service status", href: site.statusOrigin },
      { label: "Read the FAQ", href: "/faq/" },
    ],
  },
  {
    slug: "about",
    eyebrow: "About",
    title: "A native Linux home for deliberate Codex work.",
    description:
      "QuireForge is an early-stage project by James Jennison exploring coherent local project identity, observable workflows, approvals, review, and supported integrations on Linux.",
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
          "The visual identity and project direction are original. QuireForge does not redistribute OpenAI applications, imitate protected branding, or promise proprietary access without a supported interface.",
        ],
      },
      {
        heading: "Public project and source",
        paragraphs: [
          "This website presents the approved public project identity and product direction. Source code, issue tracking, milestones, and development activity are available in the public GitHub repository.",
        ],
      },
    ],
    links: [
      { label: "View source on GitHub", href: site.repository },
      { label: "James Jennison project hub", href: site.masterOrigin },
      { label: "Read the public roadmap", href: "/roadmap/" },
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
