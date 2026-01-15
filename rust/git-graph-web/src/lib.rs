use chrono::{DateTime, Local};
use git2::{Branch, BranchType, Oid, Repository, Sort};
use libcommon::{Getter, prelude::*};
use serde::Serialize;
use std::{collections::HashMap, ffi::OsStr, path::Path};

#[derive(Debug, Getter, Serialize)]
pub struct GitCommit {
    id: String,
    short_id: String,
    message: String,
    author: String,
    email: String,
    date: String,
    timestamp: i64,
    parents: Vec<String>,
    branches: Vec<String>,
    tags: Vec<String>,
}

#[derive(Debug, Getter, Serialize)]
pub struct GitBranch {
    name: String,
    is_head: bool,
    is_remote: bool,
    commit_count: usize,
}

#[derive(Getter)]
pub struct GitRepo {
    path: String,
    name: String,
    #[getter(skip)]
    repo: Repository,
    #[getter(skip)]
    branch_mapper: HashMap<Oid, Vec<String>>,
    #[getter(skip)]
    tags_mapper: HashMap<Oid, Vec<String>>,
}

impl TryFrom<&Path> for GitRepo {
    type Error = libcommon::prelude::Err;

    fn try_from(value: &Path) -> std::result::Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl GitRepo {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let repo = Repository::open(path)?;

        let path_str = path.to_string();
        let name = path
            .file_name()
            .unwrap_or(OsStr::new("unknown"))
            .to_string();

        let branch_mapper = Self::map_commit_branch(&repo, Some(BranchType::Local))?;
        let tags_mapper = Self::map_commit_tags(&repo)?;

        Ok(Self {
            path: path_str,
            name,
            repo,
            branch_mapper,
            tags_mapper,
        })
    }

    pub fn get_current_branch(&self) -> Result<String> {
        let head = self.repo.head()?;
        Ok(head.shorthand().unwrap_or("detached").to_string())
    }

    pub fn get_commits(&self, limit: Option<usize>) -> Result<Vec<GitCommit>> {
        let mut revwalk = self.repo.revwalk()?;
        revwalk.set_sorting(Sort::TIME)?;
        revwalk.push_head()?;

        let mut commits = Vec::new();

        for (index, oid) in revwalk.enumerate() {
            if let Some(limit) = limit
                && index >= limit
            {
                break;
            }

            let oid = oid?;
            let mut commit = GitCommit::try_from((oid, &self.repo))?;
            commit.branches = self.branch_mapper.get(&oid).cloned().unwrap_or_default();
            commit.tags = self.tags_mapper.get(&oid).cloned().unwrap_or_default();
            commits.push(commit);
        }

        Ok(commits)
    }

    pub fn get_all_commits(&self) -> Result<Vec<GitCommit>> {
        self.get_commits(None)
    }

    pub fn get_commits_count(&self) -> Result<usize> {
        let commits = self.get_commits(None)?;
        Ok(commits.len())
    }

    pub fn get_branches(&self, filter: Option<BranchType>) -> Result<Vec<GitBranch>> {
        let mut branches = Vec::new();
        let mut branch_names = self.repo.branches(filter)?;

        while let Some(Ok((branch, branch_type))) = branch_names.next() {
            branches.push(GitBranch::try_from((branch, branch_type, &self.repo))?);
        }

        Ok(branches)
    }

    pub fn get_branches_local(&self) -> Result<Vec<GitBranch>> {
        self.get_branches(Some(BranchType::Local))
    }

    pub fn get_tags_count(&self) -> Result<usize> {
        Ok(self.repo.tag_names(None)?.len())
    }

    pub fn get_commit_by_id(&self, commit_id: &str) -> Result<Option<GitCommit>> {
        if let Ok(oid) = Oid::from_str(commit_id)
            && let Ok(mut commit) = GitCommit::try_from((oid, &self.repo))
        {
            commit.branches = self.branch_mapper.get(&oid).cloned().unwrap_or_default();
            commit.tags = self.tags_mapper.get(&oid).cloned().unwrap_or_default();
            return Ok(Some(commit));
        }
        Ok(None)
    }

    pub fn search_commits_by_author(&self, author: &str) -> Result<Vec<GitCommit>> {
        let commits = self.get_commits(None)?;
        Ok(commits
            .into_iter()
            .filter(|commit| commit.author.contains(author) || commit.email.contains(author))
            .collect())
    }

    pub fn search_commits_by_message(&self, keyword: &str) -> Result<Vec<GitCommit>> {
        let commits = self.get_commits(None)?;
        Ok(commits
            .into_iter()
            .filter(|commit| {
                commit
                    .message
                    .to_lowercase()
                    .contains(&keyword.to_lowercase())
            })
            .collect())
    }

    pub fn get_repo_info(&self) -> Result<GitRepoInfo> {
        let commits = self.get_all_commits()?;
        let branches = self.get_branches_local()?;
        let curr_branch = self.get_current_branch()?;
        let commit_count = commits.len();
        let branch_count = branches.len();
        let tag_count = self.get_tags_count()?;

        Ok(GitRepoInfo {
            path: self.path.clone(),
            name: self.name.clone(),
            commit_count,
            branch_count,
            tag_count,
            curr_branch,
            commits,
            branches,
        })
    }

    fn map_commit_branch(
        repo: &Repository,
        filter: Option<BranchType>,
    ) -> Result<HashMap<Oid, Vec<String>>> {
        let mut map = HashMap::new();
        let branches = repo.branches(filter)?;

        for branch in branches {
            let (branch, _) = branch?;
            if let Ok(Some(name)) = branch.name()
                && let Ok(commit) = branch.get().peel_to_commit()
            {
                map.entry(commit.id())
                    .or_insert_with(Vec::new)
                    .push(name.to_string());
            }
        }
        Ok(map)
    }

    fn map_commit_tags(repo: &Repository) -> Result<HashMap<Oid, Vec<String>>> {
        let mut map = HashMap::new();
        let tags = repo.tag_names(None)?;

        for name in tags.iter().flatten() {
            if let Ok(tag_ref) = repo.find_reference(&format!("refs/tags/{}", name))
                && let Ok(commit) = tag_ref.peel_to_commit()
            {
                map.entry(commit.id())
                    .or_insert_with(Vec::new)
                    .push(name.to_string());
            }
        }
        Ok(map)
    }
}

#[derive(Debug, Getter, Serialize)]
pub struct GitRepoInfo {
    path: String,
    name: String,
    commit_count: usize,
    branch_count: usize,
    tag_count: usize,
    curr_branch: String,
    commits: Vec<GitCommit>,
    branches: Vec<GitBranch>,
}

impl TryFrom<(Branch<'_>, BranchType, &Repository)> for GitBranch {
    type Error = git2::Error;

    fn try_from(
        (branch, branch_type, repo): (Branch<'_>, BranchType, &Repository),
    ) -> std::result::Result<Self, Self::Error> {
        let name = branch.name()?.unwrap_or("").to_string();
        let is_head = branch.is_head();
        let commit = branch.get().peel_to_commit().ok();

        let commit_count = if let Some(commit) = commit {
            let mut walker = repo.revwalk()?;
            walker.push(commit.id())?;
            walker.count()
        } else {
            0
        };

        Ok(Self {
            name,
            is_head,
            is_remote: branch_type == BranchType::Remote,
            commit_count,
        })
    }
}

impl TryFrom<(Oid, &Repository)> for GitCommit {
    type Error = git2::Error;

    fn try_from((oid, repo): (Oid, &Repository)) -> std::result::Result<Self, Self::Error> {
        let id = oid.to_string();
        let short_id = id[..7].to_string();
        let commit = repo.find_commit(oid)?;

        let message = commit.message().unwrap_or("").to_string();
        let author = commit.author().name().unwrap_or("").to_string();
        let email = commit.author().email().unwrap_or("").to_string();
        let time = commit.time();
        let date_time: DateTime<Local> = DateTime::from_timestamp(time.seconds(), 0)
            .unwrap_or(Local::now().into())
            .into();
        let date = date_time.format("%Y-%m-%d %H:%M:%S").to_string();

        let parents = commit
            .parents()
            .map(|c| c.id().to_string())
            .collect::<Vec<_>>();
        let timestamp = time.seconds();

        Ok(Self {
            id,
            short_id,
            message,
            author,
            email,
            date,
            timestamp,
            parents,
            branches: vec![],
            tags: vec![],
        })
    }
}

trait StringExt {
    fn to_string(&self) -> String;
}

impl StringExt for &Path {
    fn to_string(&self) -> String {
        self.to_string_lossy().to_string()
    }
}

impl StringExt for &OsStr {
    fn to_string(&self) -> String {
        self.to_string_lossy().to_string()
    }
}
