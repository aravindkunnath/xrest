package models

type HistoryEntry struct {
	ID                 string   `json:"id" yaml:"id"`
	ServiceID          *string  `json:"serviceId,omitempty" yaml:"serviceId,omitempty"`
	EndpointID         *string  `json:"endpointId,omitempty" yaml:"endpointId,omitempty"`
	Method             string   `json:"method" yaml:"method"`
	URL                string   `json:"url" yaml:"url"`
	RequestHeaders     []Header `json:"requestHeaders" yaml:"requestHeaders"`
	RequestBody        string   `json:"requestBody" yaml:"requestBody"`
	ResponseStatus     uint16   `json:"responseStatus" yaml:"responseStatus"`
	ResponseStatusText string   `json:"responseStatusText" yaml:"responseStatusText"`
	ResponseHeaders    []Header `json:"responseHeaders" yaml:"responseHeaders"`
	ResponseBody       string   `json:"responseBody" yaml:"responseBody"`
	TimeElapsed        uint64   `json:"timeElapsed" yaml:"timeElapsed"`
	Size               uint64   `json:"size" yaml:"size"`
	CreatedAt          string   `json:"createdAt" yaml:"createdAt"`
}
