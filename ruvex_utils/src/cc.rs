use crate::errors::ConventionalCommitParseError;
use crate::SemVerChangeType;
use core::fmt;
use regex::Regex;
use ruvex_config::Config;

pub trait CCVec {
    fn is_patch(&self) -> bool;
    fn is_minor(&self) -> bool;
    fn is_major(&self) -> bool;
    fn max_change(&self) -> SemVerChangeType;
}

#[derive(Debug, PartialEq, Clone)]
pub struct ConventionalCommit {
    commit_type: String,       // feat
    short_sha: String,         //
    scope: Option<String>,     // ()
    change: SemVerChangeType,  // !
    short_description: String, // : to \n both excluded
    body: Option<String>,      // remainder of commit message
    footer: Option<String>,    // optional footer
}

// check for type in types list

impl fmt::Display for ConventionalCommit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut bc: &str = "";
        if self.change == SemVerChangeType::Major {
            bc = "!"
        }
        if let Some(ref bd) = self.body {
            write!(
                f,
                "{}{}: {}\n{}",
                self.commit_type, bc, self.short_description, bd
            )
        } else {
            write!(f, "{}{}: {}", self.commit_type, bc, self.short_description)
        }
    }
}

impl CCVec for Vec<ConventionalCommit> {
    fn is_patch(&self) -> bool {
        self.iter().all(|x| x.is_patch()) && !self.is_major()
    }
    fn is_minor(&self) -> bool {
        self.iter().any(|x| x.is_minor()) && !self.is_major()
    }
    fn is_major(&self) -> bool {
        self.iter().any(|x| x.is_major())
    }
    fn max_change(&self) -> SemVerChangeType {
        self.iter().map(|x| x.change.clone()).max().unwrap()
    }
}

impl ConventionalCommit {
    fn is_patch(&self) -> bool {
        self.change == SemVerChangeType::Minor
    }
    fn is_minor(&self) -> bool {
        self.change == SemVerChangeType::Minor
    }

    fn is_major(&self) -> bool {
        self.change == SemVerChangeType::Major
    }
    pub fn new(
        to_parse: &str,
        config: &Config,
        short_sha: String,
    ) -> Result<Self, ConventionalCommitParseError> {
        // check for :
        let mut cc_type: String;
        let mut scope: Option<String>;
        let mut change: SemVerChangeType;
        let regex: Regex = Regex::new(r"\(.+\)").unwrap();
        let short_description: String;
        let mut body: Option<String>;
        let mut footer: Option<String> = None;

        if !to_parse.contains(':') {
            return Err(ConventionalCommitParseError::MissingColumn);
        }

        if to_parse.contains("()") {
            return Err(ConventionalCommitParseError::EmptyScope);
        }

        let msg: String;
        //let (mut cc_type, msg) = to_parse.split_once(':').unwrap();
        (cc_type, msg) = match to_parse.split_once(": ") {
            Some(splitted) => {
                let (short, long) = splitted;
                (short.to_owned(), long.to_owned())
            }
            None => {
                return Err(ConventionalCommitParseError::NoSpaceAfterColumn);
            }
        };

        // check for optional scope
        scope = regex
            .captures(&cc_type)
            .map(|cap| cap.get(0).unwrap().as_str().to_owned());
        // if scope remove it from type and trim it
        if let Some(ref scope_str) = scope {
            cc_type = cc_type.replace(scope_str, "");
            scope = Some(scope_str.replace(['(', ')'], ""));
        }

        //check for ! and pop it
        if cc_type.ends_with('!') {
            change = SemVerChangeType::Major;
            cc_type = cc_type.replace('!', "");
        } else if config.minor_trigger.contains(&cc_type) {
            change = SemVerChangeType::Minor;
        } else if config.patch_trigger.contains(&cc_type) {
            change = SemVerChangeType::Patch;
        } else {
            change = SemVerChangeType::None;
        }

        // check if commit type is valid
        if !config.cc_type_in_config(&cc_type) {
            return Err(ConventionalCommitParseError::InvalidType {
                expected: config.cc_types.clone(),
                found: cc_type,
            });
        }

        (short_description, body) = match msg.split_once('\n') {
            Some(splitted) => {
                let (short, long) = splitted;
                if long.is_empty() {
                    (short.to_owned(), None)
                } else {
                    (short.to_owned(), Some(long.to_owned()))
                }
            }
            None => (msg.to_owned(), None),
        };

        // check for footer
        if let Some(ref text) = body {
            (body, footer) = match text.split_once("\n\n") {
                Some(splitted) => {
                    let (short, long) = splitted;
                    if long.is_empty() {
                        (Some(short.to_owned()), None)
                    } else {
                        (Some(short.to_owned()), Some(long.to_owned()))
                    }
                }
                None => (body, None),
            };
        }

        // check for BREAKING-CHANGE in footer
        if let Some(ref ft) = footer {
            if ft.contains("BREAKING-CHANGE: ") || ft.contains("BREAKING CHANGE: ") {
                change = SemVerChangeType::Major;
            }
        }
        Ok(ConventionalCommit {
            commit_type: cc_type,
            short_sha,
            scope,
            change,
            short_description,
            body,
            footer,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::SemVerChangeType;
    use crate::errors::ConventionalCommitParseError;
    use crate::ConventionalCommit;
    use ruvex_config::Config;
    fn test_cc(cc_message: &str, cc_types: Vec<String>, sha: &str, cc_check: ConventionalCommit) {
        let cc = ConventionalCommit::new(
            cc_message,
            &Config {
                cc_types,
                check: None,
                tag: None,
                minor_trigger: vec!["feat".to_owned()],
                patch_trigger: vec!["fix".to_owned()],
            },
            sha.to_owned(),
        )
        .unwrap();
        assert_eq!(cc, cc_check);
    }
    fn test_cc_error(
        cc_message: &str,
        cc_types: Vec<String>,
        sha: &str,
        error: ConventionalCommitParseError,
    ) {
        let cc = ConventionalCommit::new(
            cc_message,
            &Config {
                cc_types,
                check: None,
                tag: None,
                minor_trigger: vec!["feat".to_owned()],
                patch_trigger: vec!["fix".to_owned()],
            },
            sha.to_owned(),
        );
        assert_eq!(cc.unwrap_err(), error);
    }
    #[test]
    fn classic_cc() {
        let cc_message = "feat(optional scope): good commit\nthis is a classic commit";
        let cc_types: Vec<String> = ["feat"].into_iter().map(String::from).collect();
        let sha = "ababa";
        let cc = ConventionalCommit {
            commit_type: "feat".to_owned(),
            short_sha: "ababa".to_owned(),
            scope: Some("optional scope".to_owned()),
            change: SemVerChangeType::Minor,
            short_description: "good commit".to_owned(),
            body: Some("this is a classic commit".to_owned()),
            footer: None,
        };
        test_cc(cc_message, cc_types, sha, cc);
    }
    #[test]
    fn sad_brackets() {
        let cc_message = "feat(opti(onal scope): good commit\nthis is a classic commit";
        let cc_types: Vec<String> = ["feat"].into_iter().map(String::from).collect();
        let sha = "ababa";
        let cc = ConventionalCommit {
            commit_type: "feat".to_owned(),
            short_sha: "ababa".to_owned(),
            scope: Some("optional scope".to_owned()),
            change: SemVerChangeType::Minor,
            short_description: "good commit".to_owned(),
            body: Some("this is a classic commit".to_owned()),
            footer: None,
        };
        test_cc(cc_message, cc_types, sha, cc);
        let cc_message = "feat(opti(onal sco)pe): good commit\nthis is a classic commit";
        let cc_types: Vec<String> = ["feat"].into_iter().map(String::from).collect();
        let sha = "ababa";
        let cc = ConventionalCommit {
            commit_type: "feat".to_owned(),
            short_sha: "ababa".to_owned(),
            scope: Some("optional scope".to_owned()),
            change: SemVerChangeType::Minor,
            short_description: "good commit".to_owned(),
            body: Some("this is a classic commit".to_owned()),
            footer: None,
        };
        test_cc(cc_message, cc_types, sha, cc);
    }
    #[test]
    fn no_body() {
        let cc_message = "feat: good commit\n";
        let cc_types: Vec<String> = ["feat"].into_iter().map(String::from).collect();
        let sha = "ababa";
        let cc = ConventionalCommit {
            commit_type: "feat".to_owned(),
            short_sha: "ababa".to_owned(),
            scope: None,
            change: SemVerChangeType::Minor,
            short_description: "good commit".to_owned(),
            body: None,
            footer: None,
        };
        test_cc(cc_message, cc_types, sha, cc);
    }
    #[test]
    fn breaking_change_symbol() {
        let cc_message = "feat!: good commit\n";
        let cc_types: Vec<String> = ["feat"].into_iter().map(String::from).collect();
        let sha = "ababa";
        let cc = ConventionalCommit {
            commit_type: "feat".to_owned(),
            short_sha: "ababa".to_owned(),
            scope: None,
            change: SemVerChangeType::Major,
            short_description: "good commit".to_owned(),
            body: None,
            footer: None,
        };
        test_cc(cc_message, cc_types, sha, cc);
    }
    #[test]
    fn breaking_change_body() {
        let cc_message = "feat: good commit\nbodyisbody\n\nBREAKING-CHANGE: sad change";
        let cc_types: Vec<String> = ["feat"].into_iter().map(String::from).collect();
        let sha = "ababa";
        let cc = ConventionalCommit {
            commit_type: "feat".to_owned(),
            short_sha: "ababa".to_owned(),
            scope: None,
            change: SemVerChangeType::Major,
            short_description: "good commit".to_owned(),
            body: Some("bodyisbody".to_owned()),
            footer: Some("BREAKING-CHANGE: sad change".to_owned()),
        };
        test_cc(cc_message, cc_types, sha, cc);
    }
    #[test]
    fn footer() {
        let cc_message = "feat: good commit\nbodyisbody\n\nchange: sad change";
        let cc_types: Vec<String> = ["feat"].into_iter().map(String::from).collect();
        let sha = "ababa";
        let cc = ConventionalCommit {
            commit_type: "feat".to_owned(),
            short_sha: "ababa".to_owned(),
            scope: None,
            change: SemVerChangeType::Minor,
            short_description: "good commit".to_owned(),
            body: Some("bodyisbody".to_owned()),
            footer: Some("change: sad change".to_owned()),
        };
        test_cc(cc_message, cc_types, sha, cc);
    }
    #[test]
    fn missing_column() {
        let cc_message = "feat good commit\n";
        let cc_types: Vec<String> = ["feat"].into_iter().map(String::from).collect();
        let sha = "ababa";
        test_cc_error(
            cc_message,
            cc_types,
            sha,
            ConventionalCommitParseError::MissingColumn,
        );
    }
    #[test]
    fn invalid_type() {
        let cc_message = "fea: good commit\n";
        let cc_types: Vec<String> = ["feat"].into_iter().map(String::from).collect();
        let sha = "ababa";
        test_cc_error(
            cc_message,
            cc_types.clone(),
            sha,
            ConventionalCommitParseError::InvalidType {
                expected: cc_types,
                found: "fea".to_owned(),
            },
        );
    }
    #[test]
    fn empty_scope() {
        let cc_message = "feat(): good commit\n";
        let cc_types: Vec<String> = ["feat"].into_iter().map(String::from).collect();
        let sha = "ababa";
        test_cc_error(
            cc_message,
            cc_types.clone(),
            sha,
            ConventionalCommitParseError::EmptyScope,
        );
    }

    use crate::cc::CCVec;
    #[test]
    fn breaking_change_edge_cases() {
        let cc = ConventionalCommit {
            commit_type: "feat".to_owned(),
            short_sha: "ababa".to_owned(),
            scope: None,
            change: SemVerChangeType::Major,
            short_description: "good commit".to_owned(),
            body: Some("bodyisbody".to_owned()),
            footer: Some("BREAKING-CHANGE: sad change".to_owned()),
        };

        assert!(cc.is_major());
        assert!(!cc.is_minor());
        assert!(!cc.is_patch());
    }
    #[test]
    fn vector_of_cc_major() {
        let cc = ConventionalCommit {
            commit_type: "feat".to_owned(),
            short_sha: "ababa".to_owned(),
            scope: None,
            change: SemVerChangeType::Major,
            short_description: "good commit".to_owned(),
            body: Some("bodyisbody".to_owned()),
            footer: Some("BREAKING-CHANGE: sad change".to_owned()),
        };
        let cc2 = ConventionalCommit {
            commit_type: "feat".to_owned(),
            short_sha: "ababa".to_owned(),
            scope: None,
            change: SemVerChangeType::Minor,
            short_description: "good commit".to_owned(),
            body: Some("bodyisbody".to_owned()),
            footer: Some("BREAKING-CHANGE: sad change".to_owned()),
        };
        let cc3 = ConventionalCommit {
            commit_type: "fix".to_owned(),
            short_sha: "ababa".to_owned(),
            scope: None,
            change: SemVerChangeType::Minor,
            short_description: "good commit".to_owned(),
            body: Some("bodyisbody".to_owned()),
            footer: Some("test".to_owned()),
        };
        let vector: Vec<ConventionalCommit> = vec![cc, cc2, cc3];
        assert!(vector.is_major());
        assert!(!vector.is_minor());
        assert!(!vector.is_patch());
        assert!(vector.max_change() == SemVerChangeType::Major);
    }
    #[test]
    fn vector_of_cc_minor() {
        let cc = ConventionalCommit {
            commit_type: "feat".to_owned(),
            short_sha: "ababa".to_owned(),
            scope: None,
            change: SemVerChangeType::Minor,
            short_description: "good commit".to_owned(),
            body: Some("bodyisbody".to_owned()),
            footer: Some("abababa".to_owned()),
        };
        let cc2 = ConventionalCommit {
            commit_type: "fix".to_owned(),
            short_sha: "ababa".to_owned(),
            scope: None,
            change: SemVerChangeType::Patch,
            short_description: "good commit".to_owned(),
            body: Some("bodyisbody".to_owned()),
            footer: Some("test".to_owned()),
        };
        let vector: Vec<ConventionalCommit> = vec![cc, cc2];
        assert!(!vector.is_major());
        assert!(vector.is_minor());
        assert!(!vector.is_patch());
        assert!(vector.max_change() == SemVerChangeType::Minor);
    }
}
