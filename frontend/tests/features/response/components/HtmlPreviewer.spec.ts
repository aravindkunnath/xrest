import { describe, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import HtmlPreviewer from "@/features/response/components/HtmlPreviewer.vue";

describe("HtmlPreviewer.vue Behavioral Tests", () => {
    const sampleHtml = "<html><body><h1>Hello World</h1></body></html>";

    it("renders sandboxed iframe with srcdoc in Preview mode", () => {
        const wrapper = mount(HtmlPreviewer, {
            props: { html: sampleHtml },
        });
        const iframe = wrapper.find("iframe");
        expect(iframe.exists()).toBe(true);
        expect(iframe.attributes("sandbox")).toContain("allow-scripts");
        expect(iframe.attributes("srcdoc")).toBe(sampleHtml);
    });

    it("toggles between Preview and Raw Source modes", async () => {
        const wrapper = mount(HtmlPreviewer, {
            props: { html: sampleHtml },
        });
        const rawTab = wrapper.find('[data-testid="tab-raw"]');
        expect(rawTab.exists()).toBe(true);

        await rawTab.trigger("click");
        expect(wrapper.find("iframe").exists()).toBe(false);
        expect(wrapper.find("pre").text()).toContain("Hello World");

        const previewTab = wrapper.find('[data-testid="tab-preview"]');
        await previewTab.trigger("click");
        expect(wrapper.find("iframe").exists()).toBe(true);
    });
});
