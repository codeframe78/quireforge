import { describe, expect, it } from "vitest";

import {
  downloadAvailability,
  type DownloadAvailability,
  validateDownloadAvailability,
} from "./downloads";
import { footerNavigation, requiredPageSlugs, site, sitePages } from "./site";

describe("site information architecture", () => {
  it("provides every required static page exactly once", () => {
    const slugs = sitePages.map((page) => page.slug);

    expect(new Set(slugs).size).toBe(slugs.length);
    expect([...slugs].sort()).toEqual([...requiredPageSlugs].sort());
  });

  it("uses secure external links", () => {
    const pageLinks = sitePages.flatMap((page) => page.links ?? []);
    const footerLinks = footerNavigation.flatMap((group) => group.links);

    for (const link of [...pageLinks, ...footerLinks]) {
      if (link.href.startsWith("http")) {
        expect(link.href.startsWith("https://")).toBe(true);
      }
    }
  });

  it("publishes the reviewed source and contribution boundary", () => {
    const publicContent = JSON.stringify({ footerNavigation, site, sitePages });

    expect(site.repository).toBe(
      "https://github.com/James-Jennison/quireforge",
    );
    expect(publicContent).toContain(site.repository);
    expect(publicContent).toContain(`${site.repository}/issues`);
    for (const marker of ["/pulls/", "/blob/", "/tree/"]) {
      expect(publicContent).not.toContain(marker);
    }
  });

  it("limits external navigation to the approved public ecosystem", () => {
    const pageLinks = sitePages.flatMap((page) => page.links ?? []);
    const footerLinks = footerNavigation.flatMap((group) => group.links);
    const allowedHosts = new Set([
      "jamesjennison.net",
      "quireforge.jamesjennison.net",
      "status.jamesjennison.net",
    ]);
    const allowedExactUrls = new Set([
      "https://github.com/codeframe78",
      "https://github.com/James-Jennison/quireforge",
      "https://github.com/James-Jennison/quireforge/issues",
    ]);

    for (const link of [...pageLinks, ...footerLinks]) {
      if (link.href.startsWith("https://")) {
        const url = new URL(link.href);
        expect(
          allowedHosts.has(url.hostname) || allowedExactUrls.has(url.href),
        ).toBe(true);
      }
    }
  });

  it("keeps the permanent identity contract", () => {
    expect(site.name).toBe("QuireForge");
    expect(site.tagline).toBe("Build boldly. Work locally.");
    expect(site.origin).toBe("https://quireforge.jamesjennison.net");
    expect(site.securityReportUrl).toBe("https://github.com/codeframe78");
  });

  it("keeps package download data inactive before publication approval", () => {
    expect(downloadAvailability).toEqual({
      schemaVersion: 1,
      state: "unavailable",
      statusLabel: "No downloads available",
      release: null,
      plannedFormats: ["appimage", "deb"],
    });
    expect(JSON.stringify(downloadAvailability)).not.toContain("https://");
    expect(JSON.stringify(downloadAvailability)).not.toMatch(/[0-9a-f]{64}/);
  });

  it("accepts only a complete same-origin published release record", () => {
    const published: DownloadAvailability = {
      schemaVersion: 1,
      state: "published",
      statusLabel: "Beta 0.1.0-beta.1",
      plannedFormats: ["appimage", "deb"],
      release: {
        version: "0.1.0-beta.1",
        publishedAt: "2026-07-23T20:00:00Z",
        manifestUrl:
          "https://quireforge.jamesjennison.net/downloads/v0.1.0-beta.1/release-manifest.json",
        checksumUrl:
          "https://quireforge.jamesjennison.net/downloads/v0.1.0-beta.1/SHA256SUMS",
        downloads: [
          {
            format: "appimage",
            filename: "QuireForge-0.1.0-beta.1-x86_64.AppImage",
            architecture: "x86_64",
            byteSize: 83_634_680,
            sha256:
              "0a0e793815faee2c16036610afeef1c1e3912be4b40aa0f2209f3cd57bc3f56f",
            url: "https://quireforge.jamesjennison.net/downloads/v0.1.0-beta.1/QuireForge-0.1.0-beta.1-x86_64.AppImage",
          },
          {
            format: "deb",
            filename: "quireforge_0.1.0~beta.1_amd64.deb",
            architecture: "x86_64",
            byteSize: 4_467_044,
            sha256:
              "a56e894dab67e675bcbc553b9958cc884e50b73a2c0216213d22777ed47f4a18",
            url: "https://quireforge.jamesjennison.net/downloads/v0.1.0-beta.1/quireforge_0.1.0~beta.1_amd64.deb",
          },
        ],
      },
    };

    expect(validateDownloadAvailability(published, site.origin)).toEqual([]);

    const wrongOrigin = structuredClone(published);
    if (wrongOrigin.state === "published") {
      const appimage = wrongOrigin.release.downloads.find(
        (download) => download.format === "appimage",
      );
      expect(appimage).toBeDefined();
      if (!appimage) {
        throw new Error("published fixture is missing its AppImage");
      }
      appimage.url =
        "https://github.com/James-Jennison/quireforge/releases/download/v0.1.0-beta.1/QuireForge-0.1.0-beta.1-x86_64.AppImage";
    }
    expect(validateDownloadAvailability(wrongOrigin, site.origin)).toContain(
      "appimage URL is outside the approved release path",
    );

    const invalidHash = structuredClone(published);
    if (invalidHash.state === "published") {
      invalidHash.release.downloads[0]!.sha256 = "A".repeat(64);
    }
    expect(validateDownloadAvailability(invalidHash, site.origin)).toContain(
      "appimage SHA-256 is invalid",
    );

    const invalidManifest = structuredClone(published);
    if (invalidManifest.state === "published") {
      invalidManifest.release.manifestUrl =
        "https://quireforge.jamesjennison.net/downloads/release-manifest.json?version=0.1.0-beta.1";
    }
    expect(
      validateDownloadAvailability(invalidManifest, site.origin),
    ).toContain("manifest URL is outside the approved HTTPS origin");

    const missingDeb = structuredClone(published);
    if (missingDeb.state === "published") {
      missingDeb.release.downloads = missingDeb.release.downloads.filter(
        (download) => download.format !== "deb",
      );
    }
    expect(validateDownloadAvailability(missingDeb, site.origin)).toEqual(
      expect.arrayContaining([
        "missing deb download",
        "published release contains an unexpected artifact count",
      ]),
    );

    const duplicate = structuredClone(published);
    if (duplicate.state === "published") {
      duplicate.release.downloads[1]!.format = "appimage";
    }
    expect(validateDownloadAvailability(duplicate, site.origin)).toEqual(
      expect.arrayContaining([
        "duplicate appimage download",
        "missing deb download",
      ]),
    );

    const incoherentVersion = structuredClone(published);
    if (incoherentVersion.state === "published") {
      incoherentVersion.release.version = "0.1.0-beta.2";
    }
    expect(
      validateDownloadAvailability(incoherentVersion, site.origin),
    ).toEqual(
      expect.arrayContaining([
        "manifest URL is outside the approved HTTPS origin",
        "appimage filename does not match the release",
        "deb filename does not match the release",
      ]),
    );
  });
});
