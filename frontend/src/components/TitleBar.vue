<script setup lang="ts">
import { ref, onMounted } from "vue";
import { Window } from "@wailsio/runtime";

const isMaximized = ref(false);

const toggleMaximize = async () => {
    await Window.ToggleMaximise();
    isMaximized.value = await Window.IsMaximised();
};

// Function to handle double click on the title bar
const handleDoubleClick = () => {
    toggleMaximize();
};

onMounted(async () => {
    isMaximized.value = await Window.IsMaximised();
});
</script>

<template>
    <div class="titlebar" @dblclick="handleDoubleClick">
        <!-- Background drag region sits behind interactive elements -->
        <div class="titlebar-drag-region"></div>

        <!-- Left space: reserved for macOS traffic lights -->
        <div class="mac-traffic-lights-spacer"></div>

        <!-- Center navigation header -->
        <div class="titlebar-nav-header">
            <slot>
                <span class="brand-name">XREST</span>
            </slot>
        </div>

        <!-- Right space spacer to balance the mac traffic lights on the left -->
        <div class="right-spacer"></div>
    </div>
</template>

<style scoped>
.titlebar {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    height: 30px;
    background: var(--background);
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 16px;
    z-index: 1000;
    user-select: none;
    -webkit-user-select: none;
}

/* Background drag region */
.titlebar-drag-region {
    --wails-draggable: drag;
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    z-index: 1;
}

.mac-traffic-lights-spacer {
    width: 75px;
    height: 100%;
    flex-shrink: 0;
    position: relative;
    z-index: 2;
}

.right-spacer {
    width: 75px;
    height: 100%;
    flex-shrink: 0;
    position: relative;
    z-index: 2;
}

/* Navigation Header */
.titlebar-nav-header {
    position: relative;
    z-index: 2;
    display: flex;
    align-items: center;
    justify-content: center;
}

.brand-name {
    font-size: 13px;
    font-weight: 700;
    letter-spacing: 0.15em;
    color: var(--foreground);
}
</style>
