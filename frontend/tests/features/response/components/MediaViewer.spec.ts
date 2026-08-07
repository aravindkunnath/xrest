import { describe, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import MediaViewer from "@/features/response/components/MediaViewer.vue";

describe("MediaViewer.vue Behavioral Tests", () => {
    it("renders audio element for audio mime type", () => {
        const wrapper = mount(MediaViewer, {
            props: {
                src: "https://example.com/audio.mp3",
                mime: "audio/mpeg",
            },
        });
        const audio = wrapper.find("audio");
        expect(audio.exists()).toBe(true);
        expect(audio.attributes("controls")).toBeDefined();
        expect(audio.find("source").attributes("src")).toBe("https://example.com/audio.mp3");
    });

    it("renders video element for video mime type", () => {
        const wrapper = mount(MediaViewer, {
            props: {
                src: "https://example.com/video.mp4",
                mime: "video/mp4",
            },
        });
        const video = wrapper.find("video");
        expect(video.exists()).toBe(true);
        expect(video.attributes("controls")).toBeDefined();
        expect(video.find("source").attributes("src")).toBe("https://example.com/video.mp4");
    });
});
