package frontend

import "embed"

// Assets embeds the frontend/dist directory.
//
//go:embed all:dist
var Assets embed.FS
