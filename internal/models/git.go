package models


type GitStatus struct {
	IsGit                 bool   `yaml:"isGit" json:"isGit"`
	RemoteURL             string `yaml:"remoteUrl" json:"remoteUrl"`
	Branch                string `yaml:"branch" json:"branch"`
	HasUncommittedChanges bool   `yaml:"hasUncommittedChanges" json:"hasUncommittedChanges"`
	HasUnpushedCommits    bool   `yaml:"hasUnpushedCommits" json:"hasUnpushedCommits"`
	LastSync              int64  `yaml:"lastSync" json:"lastSync"`
}

