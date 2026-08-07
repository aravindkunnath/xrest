import { describe, it, expect } from "vitest";
import { resolveContentType } from "@/core/utils/contentTypeResolver";

describe("contentTypeResolver", () => {
    it("should resolve JSON MIME types", () => {
        expect(resolveContentType("application/json").category).toBe("json");
        expect(resolveContentType("application/ld+json").category).toBe("json");
        expect(resolveContentType("application/vnd.api+json").category).toBe("json");
        expect(resolveContentType("application/problem+json; charset=utf-8").category).toBe("json");
    });

    it("should resolve Image MIME types", () => {
        expect(resolveContentType("image/png").category).toBe("image");
        expect(resolveContentType("image/jpeg").category).toBe("image");
        expect(resolveContentType("image/svg+xml").category).toBe("image");
        expect(resolveContentType("image/webp").category).toBe("image");
    });

    it("should resolve HTML and XML MIME types", () => {
        expect(resolveContentType("text/html").category).toBe("html");
        expect(resolveContentType("application/xhtml+xml").category).toBe("html");
        expect(resolveContentType("application/xml").category).toBe("xml");
        expect(resolveContentType("text/xml").category).toBe("xml");
    });

    it("should resolve Audio and Video MIME types", () => {
        expect(resolveContentType("audio/mpeg").category).toBe("audio");
        expect(resolveContentType("audio/wav").category).toBe("audio");
        expect(resolveContentType("video/mp4").category).toBe("video");
        expect(resolveContentType("video/webm").category).toBe("video");
    });

    it("should resolve CSV MIME types", () => {
        expect(resolveContentType("text/csv").category).toBe("csv");
        expect(resolveContentType("text/tab-separated-values").category).toBe("csv");
    });

    it("should resolve Multipart MIME types and parse boundary parameter", () => {
        const res = resolveContentType("multipart/form-data; boundary=----WebKitFormBoundary7MA4YWxkTrZu0gW");
        expect(res.category).toBe("multipart");
        expect(res.boundary).toBe("----WebKitFormBoundary7MA4YWxkTrZu0gW");
    });

    it("should resolve PDF and Binary MIME types", () => {
        expect(resolveContentType("application/pdf").category).toBe("pdf");
        expect(resolveContentType("application/octet-stream").category).toBe("binary");
        expect(resolveContentType("application/zip").category).toBe("binary");
    });

    it("should extract charset parameter", () => {
        const res = resolveContentType("text/plain; charset=utf-8");
        expect(res.category).toBe("text");
        expect(res.charset).toBe("utf-8");
    });

    it("should fallback gracefully for empty or unknown MIME types", () => {
        expect(resolveContentType(undefined).category).toBe("text");
        expect(resolveContentType("").category).toBe("text");
        expect(resolveContentType("unknown/format").category).toBe("text");
    });
});
