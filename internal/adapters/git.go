package adapters

import (
	"time"
	"xrest/internal/models"

	"github.com/go-git/go-git/v5"
	"github.com/go-git/go-git/v5/config"
	"github.com/go-git/go-git/v5/plumbing"
	"github.com/go-git/go-git/v5/plumbing/object"
)

// GitAdapter handles Git repository operations via go-git library.
type GitAdapter struct{}

// NewGitAdapter creates a new GitAdapter instance.
func NewGitAdapter() *GitAdapter {
	return &GitAdapter{}
}

// IsRepo checks if the directory is a Git repository.
func (g *GitAdapter) IsRepo(directory string) bool {
	_, err := git.PlainOpen(directory)
	return err == nil
}

// Init initializes a Git repository in the directory.
func (g *GitAdapter) Init(directory string, remoteURL string) error {
	repo, err := git.PlainInit(directory, false)
	if err != nil {
		return err
	}

	if remoteURL != "" {
		_, err = repo.CreateRemote(&config.RemoteConfig{
			Name: "origin",
			URLs: []string{remoteURL},
		})
		if err != nil {
			return err
		}
	}

	_ = g.Commit(directory, "Initial commit from xrest")
	return nil
}

// Status retrieves the Git status of the specified directory.
func (g *GitAdapter) Status(directory string) (models.GitStatus, error) {
	repo, err := git.PlainOpen(directory)
	if err != nil {
		return models.GitStatus{IsGit: false}, nil
	}

	remoteURL := ""
	remote, err := repo.Remote("origin")
	if err == nil && len(remote.Config().URLs) > 0 {
		remoteURL = remote.Config().URLs[0]
	}

	branch := ""
	head, err := repo.Head()
	if err == nil {
		branch = head.Name().Short()
	}

	hasUncommittedChanges := false
	w, err := repo.Worktree()
	if err == nil {
		status, err := w.Status()
		if err == nil && !status.IsClean() {
			hasUncommittedChanges = true
		}
	}

	hasUnpushedCommits := false
	if head != nil && remoteURL != "" {
		remoteRef, err := repo.Reference(plumbing.NewRemoteReferenceName("origin", branch), true)
		if err == nil {
			localCommit, err := repo.CommitObject(head.Hash())
			if err == nil {
				if localCommit.Hash != remoteRef.Hash() {
					isAnc, err := isAncestor(repo, remoteRef.Hash(), localCommit.Hash)
					if err == nil && isAnc {
						hasUnpushedCommits = true
					}
				}
			}
		} else {
			// Remote ref doesn't exist yet, but we have local commits and remote is set
			hasUnpushedCommits = true
		}
	}

	return models.GitStatus{
		IsGit:                 true,
		RemoteURL:             remoteURL,
		Branch:                branch,
		HasUncommittedChanges: hasUncommittedChanges,
		HasUnpushedCommits:    hasUnpushedCommits,
		LastSync:              time.Now().Unix(),
	}, nil
}

// Commit adds all files and commits changes.
func (g *GitAdapter) Commit(directory string, message string) error {
	repo, err := git.PlainOpen(directory)
	if err != nil {
		return err
	}
	w, err := repo.Worktree()
	if err != nil {
		return err
	}
	err = w.AddWithOptions(&git.AddOptions{All: true})
	if err != nil {
		return err
	}
	_, err = w.Commit(message, &git.CommitOptions{
		Author: &object.Signature{
			Name:  "xrest App",
			Email: "info@xrest.io",
			When:  time.Now(),
		},
	})
	return err
}

// Pull pulls from the remote branch.
func (g *GitAdapter) Pull(directory string) error {
	repo, err := git.PlainOpen(directory)
	if err != nil {
		return err
	}
	w, err := repo.Worktree()
	if err != nil {
		return err
	}
	err = w.Pull(&git.PullOptions{RemoteName: "origin"})
	if err == git.NoErrAlreadyUpToDate || err == plumbing.ErrReferenceNotFound {
		return nil
	}
	return err
}

// Push pushes local commits to the remote branch.
func (g *GitAdapter) Push(directory string) error {
	repo, err := git.PlainOpen(directory)
	if err != nil {
		return err
	}
	err = repo.Push(&git.PushOptions{RemoteName: "origin"})
	if err == git.NoErrAlreadyUpToDate {
		return nil
	}
	return err
}

// Sync commits changes, pulls remote changes, and pushes local changes.
func (g *GitAdapter) Sync(directory string) error {
	_ = g.Commit(directory, "Sync point: auto-committing local changes")
	if err := g.Pull(directory); err != nil {
		return err
	}
	return g.Push(directory)
}

// isAncestor checks if ancestorHash is an ancestor of descendantHash.
func isAncestor(repo *git.Repository, ancestorHash plumbing.Hash, descendantHash plumbing.Hash) (bool, error) {
	if ancestorHash == descendantHash {
		return true, nil
	}
	visited := make(map[plumbing.Hash]bool)
	queue := []plumbing.Hash{descendantHash}
	visited[descendantHash] = true

	for len(queue) > 0 {
		currHash := queue[0]
		queue = queue[1:]

		if currHash == ancestorHash {
			return true, nil
		}

		commit, err := repo.CommitObject(currHash)
		if err != nil {
			return false, err
		}

		for _, parentHash := range commit.ParentHashes {
			if !visited[parentHash] {
				visited[parentHash] = true
				queue = append(queue, parentHash)
			}
		}
	}
	return false, nil
}
