import { describe, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import MultipartViewer from "@/features/response/components/MultipartViewer.vue";

describe("MultipartViewer.vue Behavioral Tests", () => {
    const rawMultipart =
        "--boundary123\r\nContent-Disposition: form-data; name=\"username\"\r\n\r\njohn_doe\r\n--boundary123\r\nContent-Disposition: form-data; name=\"file\"; filename=\"notes.txt\"\r\nContent-Type: text/plain\r\n\r\nFile content here\r\n--boundary123--";

    it("parses multipart body into parts", () => {
        const wrapper = mount(MultipartViewer, {
            props: {
                content: rawMultipart,
                boundary: "boundary123",
            },
        });
        const parts = wrapper.findAll('[data-testid="multipart-part"]');
        expect(parts.length).toBe(2);
        expect(parts[0].text()).toContain("username");
        expect(parts[1].text()).toContain("file");
    });
});
