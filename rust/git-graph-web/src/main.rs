use git_graph_web::GitRepo;
use libcommon::{logsetup, prelude::*};

#[logsetup("log")]
#[tokio::main]
async fn main() {
    let repo = GitRepo::new("D:\\ws\\p1").unwrap();

    for ele in repo.get_branches_local().unwrap() {
        info!("branch: {:?}", ele);
    }

    for ele in repo.get_all_commits().unwrap() {
        info!("commit: {:?}", ele);
    }
}
