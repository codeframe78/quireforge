import { describe, expect, it } from "vitest";

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

  it("keeps private source and development systems out of public content", () => {
    const publicContent = JSON.stringify({ footerNavigation, site, sitePages });

    for (const marker of [
      "github.com",
      "codeframe78",
      "James-Jennison/quireforge",
      "/issues",
      "/pulls",
      "/blob/",
      "/tree/",
    ]) {
      expect(publicContent.toLowerCase()).not.toContain(marker.toLowerCase());
    }
    expect(site).not.toHaveProperty("repository");
  });

  it("limits external navigation to the approved public ecosystem", () => {
    const pageLinks = sitePages.flatMap((page) => page.links ?? []);
    const footerLinks = footerNavigation.flatMap((group) => group.links);
    const allowedHosts = new Set([
      "jamesjennison.net",
      "quireforge.jamesjennison.net",
      "status.jamesjennison.net",
    ]);

    for (const link of [...pageLinks, ...footerLinks]) {
      if (link.href.startsWith("https://")) {
        expect(allowedHosts.has(new URL(link.href).hostname)).toBe(true);
      }
    }
  });

  it("keeps the permanent identity contract", () => {
    expect(site.name).toBe("QuireForge");
    expect(site.tagline).toBe("Build boldly. Work locally.");
    expect(site.origin).toBe("https://quireforge.jamesjennison.net");
  });
});
