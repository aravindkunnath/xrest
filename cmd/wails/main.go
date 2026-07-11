package main

import (
	"log"

	"xrest/frontend"
	"xrest/internal/xrest"

	"github.com/wailsapp/wails/v3/pkg/application"
)

func init() {
	// Register a custom event whose associated data type is string.
	application.RegisterEvent[string]("time")
}

func main() {
	// 1. Initialize core business logic (no Wails dependencies)
	greeter := xrest.NewGreeter()

	// 2. Initialize Wails adapter
	greetService := NewGreetService(greeter)
	collectionGateway := NewCollectionGateway()
	serviceGateway := NewServiceGateway()
	requestGateway := NewRequestGateway()
	secretsGateway := NewSecretsGateway()
	historyGateway := NewHistoryGateway()
	settingsGateway := NewSettingsGateway()

	// 3. Create Wails application
	app := application.New(application.Options{
		Name:        "xrest",
		Description: "Service‑First REST Client for Microservices",
		Services: []application.Service{
			application.NewService(greetService),
			application.NewService(collectionGateway),
			application.NewService(serviceGateway),
			application.NewService(requestGateway),
			application.NewService(secretsGateway),
			application.NewService(historyGateway),
			application.NewService(settingsGateway),
		},
		Assets: application.AssetOptions{
			Handler: application.AssetFileServerFS(frontend.Assets),
		},
		Mac: application.MacOptions{
			ApplicationShouldTerminateAfterLastWindowClosed: true,
		},
	})

	// Create window
	app.Window.NewWithOptions(application.WebviewWindowOptions{
		Title: "XRest",
		// Window sized to the golden ratio (1000 / 618 ≈ 1.618).
		Width:     1000,
		Height:    618,
		Frameless: false,
		Mac: application.MacWindow{
			InvisibleTitleBarHeight: 50,
			Backdrop:                application.MacBackdropTranslucent,
			TitleBar: application.MacTitleBar{
				AppearsTransparent: true,
				HideTitle:          true,
				FullSizeContent:    true,
			},
		},
		BackgroundColour: application.NewRGB(6, 7, 15),
		URL:              "/",
	})

	// Run Wails
	err := app.Run()
	if err != nil {
		log.Fatal(err)
	}
}
