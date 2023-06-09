use anyhow::Result;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use std::path::Path;

#[derive(Default, Clone)]
pub struct Matcher(Option<MatcherKind>);
//
#[derive(Clone)]
pub enum MatcherKind {
  Exclude(Gitignore),

}

impl Matcher {
  pub fn new(matcher: MatcherKind) -> Self {
     Self(Some(matcher))
  }

 pub fn create_matcher(project_dir: &Path, rules: Vec<String>) -> Result<Gitignore> {

    let mut builder = GitignoreBuilder::new(project_dir);

    for rule in rules {
      builder.add_line(None, &rule)?;
    }
    Ok(builder.build()?)
  }

  pub fn should_include(&self, relative_path: &Path) -> bool {
    match &self.0 {
      Some(MatcherKind::Exclude(it)) => {
        it.matched(relative_path, /* is_dir */ false)
      }
      None => false,
    }
  }
}
