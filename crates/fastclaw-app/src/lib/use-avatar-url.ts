import { useState, useEffect, useRef } from "react";
import * as transport from "./transport";

function mimeFromExt(path: string): string {
  const ext = path.split(".").pop()?.toLowerCase();
  if (ext === "jpg" || ext === "jpeg") return "image/jpeg";
  if (ext === "webp") return "image/webp";
  return "image/png";
}

export async function loadAvatarBlobUrl(filePath: string): Promise<string | null> {
  if (!transport.isTauri || !filePath) return null;
  try {
    const { readFile } = await import("@tauri-apps/plugin-fs");
    const bytes = await readFile(filePath);
    const blob = new Blob([bytes], { type: mimeFromExt(filePath) });
    return URL.createObjectURL(blob);
  } catch {
    try {
      const { convertFileSrc } = await import("@tauri-apps/api/core");
      return convertFileSrc(filePath);
    } catch {
      return null;
    }
  }
}

/**
 * Resolves a local file path to a displayable blob: URL.
 * Automatically cleans up the previous blob URL on path change or unmount.
 */
export function useAvatarUrl(filePath: string | undefined | null): string | undefined {
  const [url, setUrl] = useState<string | undefined>(undefined);
  const blobRef = useRef<string | null>(null);

  useEffect(() => {
    if (blobRef.current) {
      URL.revokeObjectURL(blobRef.current);
      blobRef.current = null;
    }
    setUrl(undefined);

    if (!filePath) return;
    let cancelled = false;
    loadAvatarBlobUrl(filePath).then((result) => {
      if (cancelled || !result) return;
      blobRef.current = result;
      setUrl(result);
    });
    return () => { cancelled = true; };
  }, [filePath]);

  useEffect(() => {
    return () => {
      if (blobRef.current) URL.revokeObjectURL(blobRef.current);
    };
  }, []);

  return url;
}
