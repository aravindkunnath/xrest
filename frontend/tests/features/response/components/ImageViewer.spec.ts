import { describe, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import ImageViewer from "@/features/response/components/ImageViewer.vue";

describe("ImageViewer.vue Behavioral Tests", () => {
    it("renders image tag with correct src", () => {
        const wrapper = mount(ImageViewer, {
            props: {
                src: "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==",
                mime: "image/png",
            },
        });
        const img = wrapper.find("img");
        expect(img.exists()).toBe(true);
        expect(img.attributes("src")).toBe("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==");
    });

    it("displays zoom in, zoom out, and reset controls", async () => {
        const wrapper = mount(ImageViewer, {
            props: {
                src: "https://example.com/test.png",
                mime: "image/png",
            },
        });
        expect(wrapper.find('[data-testid="zoom-in"]').exists()).toBe(true);
        expect(wrapper.find('[data-testid="zoom-out"]').exists()).toBe(true);
        expect(wrapper.find('[data-testid="zoom-reset"]').exists()).toBe(true);
    });

    it("toggles canvas background mode when background button is clicked", async () => {
        const wrapper = mount(ImageViewer, {
            props: {
                src: "https://example.com/test.png",
                mime: "image/png",
            },
        });
        const bgBtn = wrapper.find('[data-testid="bg-toggle"]');
        expect(bgBtn.exists()).toBe(true);
        await bgBtn.trigger("click");
        expect(wrapper.find('[data-testid="image-canvas"]').classes()).toContain("bg-dark");
    });

    it("renders SVG raw code toggle for image/svg+xml MIME", async () => {
        const svgCode = '<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"><rect width="10" height="10"/></svg>';
        const wrapper = mount(ImageViewer, {
            props: {
                src: `data:image/svg+xml;utf8,${encodeURIComponent(svgCode)}`,
                mime: "image/svg+xml",
                rawCode: svgCode,
            },
        });
        const toggleBtn = wrapper.find('[data-testid="svg-code-toggle"]');
        expect(toggleBtn.exists()).toBe(true);
        await toggleBtn.trigger("click");
        expect(wrapper.find("pre").text()).toContain("<svg");
    });
});
