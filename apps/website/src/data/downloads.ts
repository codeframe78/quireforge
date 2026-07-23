export type DownloadFormat = "appimage" | "deb";

export type PublishedDownload = {
  format: DownloadFormat;
  filename: string;
  architecture: "x86_64";
  byteSize: number;
  sha256: string;
  url: string;
};

export type PublishedRelease = {
  version: string;
  publishedAt: string;
  manifestUrl: string;
  checksumUrl: string;
  downloads: PublishedDownload[];
};

export type DownloadAvailability =
  | {
      schemaVersion: 1;
      state: "unavailable";
      statusLabel: "No downloads available";
      release: null;
      plannedFormats: readonly DownloadFormat[];
    }
  | {
      schemaVersion: 1;
      state: "published";
      statusLabel: string;
      release: PublishedRelease;
      plannedFormats: readonly DownloadFormat[];
    };

export const downloadAvailability: DownloadAvailability = {
  schemaVersion: 1,
  state: "unavailable",
  statusLabel: "No downloads available",
  release: null,
  plannedFormats: ["appimage", "deb"],
};
