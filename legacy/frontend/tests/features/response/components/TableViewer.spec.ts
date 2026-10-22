import { describe, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import TableViewer from "@/features/response/components/TableViewer.vue";

describe("TableViewer.vue Behavioral Tests", () => {
    const csvContent = "Name,Age,Role\nAlice,30,Developer\nBob,25,Designer";

    it("parses CSV content into table headers and rows", () => {
        const wrapper = mount(TableViewer, {
            props: { content: csvContent },
        });
        const headers = wrapper.findAll("th");
        expect(headers.length).toBe(3);
        expect(headers[0].text()).toBe("Name");
        expect(headers[1].text()).toBe("Age");
        expect(headers[2].text()).toBe("Role");

        const rows = wrapper.findAll("tbody tr");
        expect(rows.length).toBe(2);
        expect(rows[0].text()).toContain("Alice");
        expect(rows[1].text()).toContain("Bob");
    });

    it("filters rows based on search query", async () => {
        const wrapper = mount(TableViewer, {
            props: { content: csvContent },
        });
        const input = wrapper.find('input[type="text"]');
        await input.setValue("Alice");
        const rows = wrapper.findAll("tbody tr");
        expect(rows.length).toBe(1);
        expect(rows[0].text()).toContain("Alice");
    });

    it("toggles sorting when column header is clicked", async () => {
        const wrapper = mount(TableViewer, {
            props: { content: csvContent },
        });
        const nameHeader = wrapper.findAll("th")[0];
        await nameHeader.trigger("click");
        const rows = wrapper.findAll("tbody tr");
        expect(rows[0].text()).toContain("Alice");
    });
});
