import { describe, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import BinaryViewer from "@/features/response/components/BinaryViewer.vue";

describe("BinaryViewer.vue Behavioral Tests", () => {
    it("renders file details card and download button", () => {
        const wrapper = mount(BinaryViewer, {
            props: {
                size: "1.4 MB",
                mime: "application/pdf",
                filename: "document.pdf",
            },
        });
        expect(wrapper.text()).toContain("document.pdf");
        expect(wrapper.text()).toContain("1.4 MB");
        expect(wrapper.text()).toContain("application/pdf");
        expect(wrapper.find('[data-testid="download-btn"]').exists()).toBe(true);
    });
});
