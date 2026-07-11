package main

import (
	"log"
	"xrest/internal/adapters"
	"xrest/internal/models"
)

// RequestGateway handles sending HTTP requests from the frontend.
type RequestGateway struct{}

// NewRequestGateway creates a new RequestGateway.
func NewRequestGateway() *RequestGateway {
	return &RequestGateway{}
}

// Send executes the given HTTP request and returns the response.
func (g *RequestGateway) Send(req *models.Request) (*models.Response, error) {
	log.Printf("[RequestGateway] Sending request: %s %s\n", req.Method, req.URL)
	client := &adapters.Http{}
	resp, err := client.Send(req)
	if err != nil {
		log.Printf("[RequestGateway] Request failed: %v\n", err)
		errStr := err.Error()
		return &models.Response{
			StatusCode: 0,
			StatusText: "Error",
			Error:      &errStr,
		}, nil
	}
	return resp, nil
}
