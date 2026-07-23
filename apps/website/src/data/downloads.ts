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

const SHA256_PATTERN = /^[0-9a-f]{64}$/;
const VERSION_PATTERN =
  /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-[0-9A-Za-z.-]+)?$/;
const UTC_TIMESTAMP_PATTERN =
  /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d{3})?Z$/;
const APPROVED_FORMATS: readonly DownloadFormat[] = ["appimage", "deb"];

function parseApprovedUrl(value: string, approvedOrigin: string): URL | null {
  try {
    const url = new URL(value);
    if (
      url.protocol !== "https:" ||
      url.origin !== approvedOrigin ||
      url.username ||
      url.password ||
      url.search ||
      url.hash
    ) {
      return null;
    }
    return url;
  } catch {
    return null;
  }
}

export function validateDownloadAvailability(
  availability: DownloadAvailability,
  approvedOrigin: string,
): string[] {
  if (availability.state === "unavailable") {
    return availability.release === null
      ? []
      : ["unavailable release must be null"];
  }

  const errors: string[] = [];
  const { release } = availability;
  if (!VERSION_PATTERN.test(release.version)) {
    errors.push("release version is invalid");
  }
  if (
    !UTC_TIMESTAMP_PATTERN.test(release.publishedAt) ||
    Number.isNaN(Date.parse(release.publishedAt)) ||
    new Date(release.publishedAt).toISOString().replace(".000Z", "Z") !==
      release.publishedAt.replace(".000Z", "Z")
  ) {
    errors.push("published time must be a UTC ISO timestamp");
  }
  const releasePath = `/downloads/v${release.version}/`;
  const manifestUrl = parseApprovedUrl(release.manifestUrl, approvedOrigin);
  if (
    !manifestUrl ||
    manifestUrl.pathname !== `${releasePath}release-manifest.json`
  ) {
    errors.push("manifest URL is outside the approved HTTPS origin");
  }
  const checksumUrl = parseApprovedUrl(release.checksumUrl, approvedOrigin);
  if (!checksumUrl || checksumUrl.pathname !== `${releasePath}SHA256SUMS`) {
    errors.push("checksum URL is outside the approved HTTPS origin");
  }

  const expectedFilenames: Record<DownloadFormat, string> = {
    appimage: `QuireForge-${release.version}-x86_64.AppImage`,
    deb: `quireforge_${release.version.replace("-", ".")}_amd64.deb`,
  };
  const seen = new Set<DownloadFormat>();
  for (const download of release.downloads) {
    if (seen.has(download.format)) {
      errors.push(`duplicate ${download.format} download`);
    }
    seen.add(download.format);
    if (download.filename !== expectedFilenames[download.format]) {
      errors.push(`${download.format} filename does not match the release`);
    }
    if (download.architecture !== "x86_64") {
      errors.push(`${download.format} architecture is unsupported`);
    }
    if (!Number.isSafeInteger(download.byteSize) || download.byteSize <= 0) {
      errors.push(`${download.format} byte size is invalid`);
    }
    if (!SHA256_PATTERN.test(download.sha256)) {
      errors.push(`${download.format} SHA-256 is invalid`);
    }
    const url = parseApprovedUrl(download.url, approvedOrigin);
    if (!url || url.pathname !== `${releasePath}${download.filename}`) {
      errors.push(
        `${download.format} URL is outside the approved release path`,
      );
    }
  }
  if (
    availability.plannedFormats.length !== APPROVED_FORMATS.length ||
    availability.plannedFormats.some(
      (format, index) => format !== APPROVED_FORMATS[index],
    )
  ) {
    errors.push("planned formats do not match the release contract");
  }
  for (const format of APPROVED_FORMATS) {
    if (!seen.has(format)) {
      errors.push(`missing ${format} download`);
    }
  }
  if (release.downloads.length !== APPROVED_FORMATS.length) {
    errors.push("published release contains an unexpected artifact count");
  }
  return errors;
}

export function getPublishedRelease(
  availability: DownloadAvailability,
): PublishedRelease | null {
  return availability.state === "published" ? availability.release : null;
}

export const downloadAvailability: DownloadAvailability = {
  schemaVersion: 1,
  state: "unavailable",
  statusLabel: "No downloads available",
  release: null,
  plannedFormats: ["appimage", "deb"],
};
