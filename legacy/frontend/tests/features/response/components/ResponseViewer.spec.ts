import { describe, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import ResponseViewer from "@/features/response/components/ResponseViewer.vue";

describe("ResponseViewer.vue Behavioral Auto-detection Tests", () => {
    const baseResponse = {
        activeTab: "body",
        status: 200,
        statusText: "200 OK",
        time: "120ms",
        size: "4.2 KB",
        type: "",
        body: "",
        error: "",
        headers: [{ name: "Content-Type", value: "text/plain" }],
        requestHeaders: [],
    };

    it("auto-detects and mounts ImageViewer for image/png response", () => {
        const response = {
            ...baseResponse,
            headers: [{ name: "Content-Type", value: "image/png" }],
            body: "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==",
        };
        const wrapper = mount(ResponseViewer, {
            props: {
                response,
                url: "http://example.com/api",
                variables: {},
                environmentName: "DEV",
            },
        });
        expect(wrapper.findComponent({ name: "ImageViewer" }).exists()).toBe(true);
    });

    it("auto-detects and mounts TableViewer for text/csv response", () => {
        const response = {
            ...baseResponse,
            headers: [{ name: "Content-Type", value: "text/csv" }],
            body: "col1,col2\nval1,val2",
        };
        const wrapper = mount(ResponseViewer, {
            props: {
                response,
                url: "http://example.com/api",
                variables: {},
                environmentName: "DEV",
            },
        });
        expect(wrapper.findComponent({ name: "TableViewer" }).exists()).toBe(true);
    });

    it("auto-detects and mounts BinaryViewer for application/octet-stream response", () => {
        const response = {
            ...baseResponse,
            headers: [{ name: "Content-Type", value: "application/octet-stream" }],
            body: "RAW_BYTES",
        };
        const wrapper = mount(ResponseViewer, {
            props: {
                response,
                url: "http://example.com/api",
                variables: {},
                environmentName: "DEV",
            },
        });
        expect(wrapper.findComponent({ name: "BinaryViewer" }).exists()).toBe(true);
    });

    it("renders empty state placeholder when response is empty and does not show Content-Type unknown", () => {
        const emptyResponse = {
            activeTab: "body",
            status: 0,
            statusText: "",
            time: "0ms",
            size: "0 B",
            type: "",
            body: "",
            error: "",
            headers: [],
            requestHeaders: [],
        };
        const wrapper = mount(ResponseViewer, {
            props: {
                response: emptyResponse,
                url: "http://example.com/api",
                variables: {},
                environmentName: "DEV",
            },
        });
        expect(wrapper.text()).toContain("Send a request to see the response");
        expect(wrapper.text()).not.toContain("Content-Type: unknown");
        expect(wrapper.find("button").exists()).toBe(false);
    });
});
