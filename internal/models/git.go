package models

import "time"

type GitStatus struct {
	IsGit      bool      `yaml:"isGit" json:"isGit"`
	RemoteURL  string    `yaml:"remoteUrl" json:"remoteUrl"`
	Branch     string    `yaml:"branch" json:"branch"`
	IsDirty    bool      `yaml:"IsDirty" json:"IsDirty"`
	DoPull     bool      `yaml:"doPull" json:"doPull"`
	DoPush     bool      `yaml:"doPush" json:"doPush"`
	LastSyncAt time.Time `yaml:"lastSyncAt" json:"lastSyncAt"`
}
