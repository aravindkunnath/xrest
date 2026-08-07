export type ResponseCategory =
    | "json"
    | "xml"
    | "html"
    | "image"
    | "audio"
    | "video"
    | "pdf"
    | "csv"
    | "multipart"
    | "binary"
    | "text";

export interface ResolvedContentType {
    category: ResponseCategory;
    mime: string;
    boundary?: string;
    charset?: string;
}

export function resolveContentType(contentTypeHeader?: string): ResolvedContentType {
    if (!contentTypeHeader) {
        return { category: "text", mime: "text/plain" };
    }

    const parts = contentTypeHeader.split(";").map((p) => p.trim());
    const rawMime = parts[0]?.toLowerCase() || "";

    let boundary: string | undefined;
    let charset: string | undefined;

    for (let i = 1; i < parts.length; i++) {
        const param = parts[i];
        if (param.toLowerCase().startsWith("boundary=")) {
            boundary = param.substring("boundary=".length).replace(/^["']|["']$/g, "");
        } else if (param.toLowerCase().startsWith("charset=")) {
            charset = param.substring("charset=".length).replace(/^["']|["']$/g, "");
        }
    }

    if (rawMime === "application/json" || rawMime.endsWith("+json")) {
        return { category: "json", mime: rawMime, boundary, charset };
    }

    if (rawMime === "image/svg+xml") {
        return { category: "image", mime: rawMime, boundary, charset };
    }

    if (rawMime.startsWith("image/")) {
        return { category: "image", mime: rawMime, boundary, charset };
    }

    if (rawMime === "text/html" || rawMime === "application/xhtml+xml") {
        return { category: "html", mime: rawMime, boundary, charset };
    }

    if (rawMime === "application/xml" || rawMime === "text/xml" || rawMime.endsWith("+xml")) {
        return { category: "xml", mime: rawMime, boundary, charset };
    }

    if (rawMime.startsWith("audio/")) {
        return { category: "audio", mime: rawMime, boundary, charset };
    }

    if (rawMime.startsWith("video/")) {
        return { category: "video", mime: rawMime, boundary, charset };
    }

    if (rawMime === "text/csv" || rawMime === "text/tab-separated-values") {
        return { category: "csv", mime: rawMime, boundary, charset };
    }

    if (rawMime.startsWith("multipart/")) {
        return { category: "multipart", mime: rawMime, boundary, charset };
    }

    if (rawMime === "application/pdf") {
        return { category: "pdf", mime: rawMime, boundary, charset };
    }

    if (
        rawMime === "application/octet-stream" ||
        rawMime.startsWith("application/zip") ||
        rawMime.startsWith("application/x-") ||
        rawMime.startsWith("application/gzip")
    ) {
        return { category: "binary", mime: rawMime, boundary, charset };
    }

    return { category: "text", mime: rawMime || "text/plain", boundary, charset };
}
